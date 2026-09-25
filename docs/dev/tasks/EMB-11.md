# EMB-11 — asistente de instalación con defaults (Q1+Q2 owner)

> **Plan:** `docs/dev/plans/2026-09-16-embeddings-auto.md` (Wave1, instalador)
> **Estado:** ⏳ IN PROGRESS (DISCOVERY completo 2026-09-16)
> **Appetite:** 1d · 🟡 · 🟠
> **Branch/Commit:** develop / `feat: EMB-11`
> **Cynefin:** 🟨 complicado (UX + Q3) · ⬆️ 1 / ⬇️ 3 steps
> **Ruta:** vanta-worker
> **SDP:** campaign-executor, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, api-and-interface-design, security-and-hardening (frontend-ui-engineering sugerida por scorer pero DESCARTADA: sin `web/`, scope discipline; writing-guidelines como referencia, no skill, para no exceder ≤8)

## 1. TAREA

**Objetivo:** asistente `setup-embeddings.ps1` que deja a un usuario nuevo con modelo local funcionando tras Enter-Enter, sin fricción de paths/vars/features y sin riesgo de filtrar secrets a disco.

**Contrato exacto (ley):**
1. `setup-embeddings.ps1` NUEVO (raíz del repo) con `-NonInteractive` que deja todo por default: provider `local`, modelo default `multilingual-e5-small`, `VANTADB_LOCAL_MODEL` absoluta de sesión.
2. `-NonInteractive` termina con modelo default presente en disco y verificado (`python embeddings/download.py --check` exit 0).
3. Modo interactivo: prompts claros por modelo / carpeta / descarga; Enter = auto (default), todo configurable (Q1).
4. Q2: los 3 modelos ya en disco se ofrecen directo; el resto del manifest SOLO se descarga avisando tamaño total + tiempo estimado + confirmación explícita antes de bajar.
5. Q3: `ollama`/`openai` se ofrecen marcando requisitos (servidor corriendo en `localhost:11434` / key en env de sesión), sin instalar nada externo.
6. NUNCA escribe secrets a disco: keys solo env de sesión (`$env:`), jamás `Set-Content`/`Add-Content`/`Out-File` con valores de key, jamás loguea valores (mask `***set***`/`not set`).
7. RESTRICCIÓN EMB-10: onnxruntime nativo ≥1.27 OBLIGATORIO (ORT_API_VERSION=27 compilado; System32 trae 1.17.1 que ABORTA; el ORT 1.30 de `Temp/` es efímero) → el wizard garantiza/descarga nativo ≥1.27 a ubicación persistente (`LOCALAPPDATA\VantaDB\onnxruntime`, fuera del repo) y setea `ORT_DYLIB_PATH` en sesión; documentado en el propio script.

**Acceptance del plan:** Q1/Q2 owner (auto con Enter, configurable; 3 verificados + resto avisado con tamaño/tiempo). Instala → corre asistente → Enter Enter → modelo listo + mensaje de éxito con dim/idioma. Agente futuro encuentra modelo configurado sin intervención.

## 2. ARCHIVOS

**Clave (leer completo antes de ACT — Regla 0):**
- `setup-embeddings.ps1` (NUEVO, raíz; NO existe — verificado `Test-Path` False) — el entregable.
- `embeddings/download.py` (264L, LEÍDO COMPLETO: `--check` valida manifest+lock sin red; `--only` por modelo; skip idempotente si `*.onnx` + pesos presentes; `ALLOW_PATTERNS` recortados FIND-71).
- `embeddings/manifest.json` (115L, LEÍDO COMPLETO: 9 modelos, `default: multilingual-e5-small`, dims 384/512/768/1024/4096, `size_onnx_mb`/`size_hf_mb` por modelo, qwen3 `onnx:null` + `exception`).
- `embeddings/verify.py` (239L, LEÍDO COMPLETO: `--check` solo estructura sin modelos ni red; `verify_model` abre sesión ONNX + tokenizer si descargado, SKIP≠FAIL; cosine thresholds son hints, no asserts).

**Relacionados:**
- `docs/dev/tasks/EMB-10.md` (evidencia build: ORT_API_VERSION=27, `ORT_DYLIB_PATH` soportado por ort `lib.rs:224`, ORT 1.30.0 en `Temp/opencode/ort130` efímero, FIND-100 abort ort).
- `src/config.rs:935-940` (`VANTADB_LOCAL_MODEL`, default `embeddings/models/multilingual-e5-small/onnx`, acepta absoluta) y `:954-959` (`VANTADB_EMBEDDING_PROVIDER`, default `ollama`).
- `embeddings/README.md` (tabla 9 modelos con tamaños/totales — fuente de texto UX del wizard).
- `docs/dev/Backlog.md` FIND-100 (abort ort <1.27 — el wizard es su mitigación lado-instalador).

**Prohibidos (NO TOCAR):** `.opencode/`, `Justfile`, `completions/_vanta-cli*`, `desktop/src-tauri/Cargo.lock`, `ocr-delegate.yml`, `ocr-review.ps1`, `reparacion.bat`, `docs/pipeline-state.json`, plan file (solo recitation orquestador), `stash@{0..14}`, archivos de EMB-12 (`vanta-mcp-local.ps1` — paralela disjunta, verificado NO existe), `Cargo.toml`, código Rust, `docs/dev/Backlog.md` y `docs/dev/avance/` (NO tocar por orden), `embeddings/models/` (solo lectura), `embeddings/*.py`, `manifest.json/lock`.

## 3. DEPENDENCIAS

- **Wave1**, paralela con EMB-12 (disjunta: EMB-12 crea `vanta-mcp-local.ps1`, este task `setup-embeddings.ps1` — 0 archivos compartidos; EMB-12 consume lo que el wizard deja vía env).
- **Hereda (Wave0 EMB-10 ✅ `a0f65d29`):** binario con motor, restricción ORT ≥1.27 (nueva, §1.7), `download.py --check`, manifest ids/dims/sizes, `verify.py`, FIND-71 (patterns smoke).
- **Sin bloqueantes.** Next: Wave2 (EMB-13/16/18).
- **Q3 local+ollama/openai-marcados:** ollama/openai solo ofrecidos con requisitos, sin instalar nada externo.

## 4. REFERENCIAS

- `.opencode/references/clean-code-clean-architecture.md` Apéndice V (LEÍDO COMPLETO: script = borde exterior/composición-drivers → Humble Object, cero lógica de negocio; SLAP, funciones chicas, nombres sin stuttering; severidades /cleanCA).
- `writing-guidelines` (referencia, mensajes claros del wizard — NO cargada como skill por tope ≤8).
- `security-and-hardening` (CARGADA: secrets NUNCA a disco — checklist §Secrets Management; threat model: trust boundary = prompt de key / env; abuse case = key persistida o logueada; auditar en review con `git diff --cached | grep -i key`).
- FIND-71 patterns smoke (`download.py --check`).
- Notion (Paso 0c, 4 páginas LEÍDAS COMPLETAS vía fetch): Problema (dimensión semántica garbage-in=garbage-out → justifica el plan) + Propuesta (retrieval híbrido REAL; `extract_skills`/gobernanza PROPUESTA — fuera de scope). Nuevas features / Plan de accion: índice sin mapeo a EMB-11 → no aplican (filtro VantaDB).
- **Spec (scripts: N/A símbolos públicos, solo UX texto — tabla de decisiones):**

| Decisión | Opción | Evidencia |
|---|---|---|
| Ubicación | raíz `setup-embeddings.ps1` | plan: "raíz o dev-tools/"; raíz más descubrible, 1-liner del contrato |
| Presencia por modelo | `onnx/model.onnx` (o `model_int8.onnx` bge-m3) + `tokenizer.json` presentes | espejo de `download.py:212` skip + `verify.py:154-163`; qwen3 `onnx:null` → nunca "presente" local |
| Default | `multilingual-e5-small`, 384d, ES+EN 16+ | `manifest.json:3,77-88`; `config.rs:937` default path |
| Estimación tiempo | tamaño total MB / 10 MB/s, redondeo a min | heurística honesta marcada como aproximada (Regla 11: sin benchmarks falsos) |
| ORT persistente | `LOCALAPPDATA\VantaDB\onnxruntime` (fuera del repo, sin tocar `.gitignore`) | `Temp/` efímero (EMB-10); repo-local contaminaría `git status`; per-user correcto para dll |
| ORT release | `onnxruntime-win-x64-1.30.0.zip` de GitHub microsoft/onnxruntime | misma versión que EMB-10 verificó en vivo (≥1.27 ✓) |
| Secrets | sin parámetro de key; lee `$env:VANTADB_OPENAI_API_KEY`/`OPENAI_API_KEY`, mask en output | contrato NUNCA-a-disco; leer≠pedir≠persistir |
| Sin símbolos públicos nuevos | N/A — script PS1, 0 Rust tocado | Gate D no dispara `question` (ver §Spec Gate) |

## 5. SKILLS

**SDP (campaign_discover_skills_v2 BUILD keywords setup-wizard/powershell-script/model-download/secrets-handling, ≤8):**
campaign-executor, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, api-and-interface-design (+ security-and-hardening por contrato secrets; frontend-ui-engineering DESCARTADA: sin `web/`).
**Base sesión:** campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail(full).
**Cargadas:** incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, api-and-interface-design, security-and-hardening.
**doubt-driven:** claim crítico = "el script nunca persiste ni loguea secrets" → reconciliación adversarial en §Ejecución (single-model; cross-model skipped: non-interactive + sin CLI externo autorizado).

## 6. HERRAMIENTAS+MCP

- RED: `pwsh -NoProfile -File setup-embeddings.ps1 -NonInteractive` (antes de crear: falla "no se reconoce como script" ✅ registrado).
- GREEN por slice: `pwsh -NoProfile -NoLogo -Command "[System.Management.Automation.Language.Parser]::ParseFile(...)"` (parse) + corrida `-NonInteractive` real.
- `python embeddings/download.py --check` (contrato: exit 0 al cerrar; verificado OK en DISCOVERY: manifest v1 9 modelos + lock 3 locked).
- `git diff --check` + `git status --short` (alcance: solo `setup-embeddings.ps1` + este task file).
- `campaign_verify_cmd` (bug exit -1 conocido → bash directa + mención en RESULTADO).
- Sin red salvo descargas opcionales confirmadas (Q2: resto avisado; ORT: confirmada en interactivo, automática en NonInteractive). Esta sesión: 0 descargas ejecutadas (todo presente) → investigación internet N/A.
- OCR: `pwsh dev-tools/ocr-review.ps1` (advisory) al cierre.

## 7. INVESTIGACIÓN CÓDIGO (DISCOVERY)

**Blast radius wizard→download.py→manifest→verify:**
- Wizard NO importa código Python; orquesta por CLI (`python embeddings/download.py --only <id> | --check`) y chequea presencia por filesystem (onnx+tokenizer). Acoplamiento = formato CLI + layout `models/<id>/`, ambos estables por contrato EMB-01.
- `download.py --check` valida manifest (9 modelos, rev 7 chars, dims, grupos 3/3/3, qwen3 exception+onnx null) + lock subset repo/rev — el wizard lo usa como verificación final sin red.
- Cómo verifica presencia cada modelo: `models/<id>/onnx/model.onnx` (bge-m3: `model_int8.onnx`) + `tokenizer.json` (raíz del modelo o recursivo) — espejo de `verify.py:154-174`.
- Modelo default e5-small: 384d, `onnx/model.onnx` 470MB + `tokenizer.json` 17MB PRESENTES (verificado en disco).
- En disco completos (onnx+tokenizer): all-MiniLM-L6-v2, multilingual-e5-small, paraphrase-multilingual-MiniLM-L12-v2 — los 3 Q2, todos 384d.
- `VANTADB_LOCAL_MODEL` acepta absoluta (`config.rs:936`); default relativo frágil por CWD → el wizard setea ABSOLUTA de sesión.
- ORT: compilado API v27 (`cargo tree` EMB-10); System32 1.17.1 ABORTA (`expect` + mutex poisoned → FIND-100); `Temp/opencode/ort130` con `onnxruntime.dll` 16.4MB existe (efímero — el wizard lo usa como cache oportunista, nunca como ubicación final).
- `codegraph_explore` devolvió ruido (vectorstore langchain, sin símbolos embeddings/) — blast radius real = 1 archivo nuevo + 0 callers (script standalone invocado por humano/CI). Cobertura N/A: nada que indexar aún.

**Impacto mapeado (Regla 0):**
- Archivos leídos completos: `embeddings/download.py`, `embeddings/verify.py`, `embeddings/manifest.json`, `embeddings/README.md`, `docs/dev/tasks/EMB-10.md`, plan EMB-11 (§Wave1), `src/config.rs:925-959`, clean-code Apéndice V, definition-of-done, Notion ×4.
- Referencias hacia dentro (EMB-11 depende de): `download.py --check/--only` (CLI), `manifest.json` (tabla UX + default), `verify.py --check`, `config.rs` (nombres vars), ORT 1.30 Temp (cache oportunista).
- Referencias entrantes (dependen de EMB-11): EMB-12 (consume env que el wizard deja); EMB-19 (verificación e2e).
- Veredicto: 1 archivo NUEVO, 0 modificaciones a código existente; impacto = aditivo puro, rollback = borrar el .ps1. Riesgo: descarga pesada (mitigado Q2 confirmación) + secreto a disco (mitigado por diseño sin-parámetro + review).

## 8. INVESTIGACIÓN PROBLEMA

- Instalar = fricción (paths relativos frágiles por CWD, features, vars con default `ollama` sin servidor, ORT nativo invisible) → wizard con defaults la elimina: Enter-Enter deja `local` + default + `ORT_DYLIB_PATH` en sesión.
- Secreto a disco = incidente (secret commiteado se rota, no se borra — security-and-hardening) → el diseño ni siquiera acepta key por parámetro: solo lee env de sesión, maskea output, 0 writes de valores.
- Descargar 16GB (qwen3) o 3.47GB (bge-m3) sin aviso = incidente de disco (precedente FIND-94) → tabla tamaño/tiempo + confirmación por modelo; qwen3 se ofrece marcado `GPU-only, sin ONNX` y nunca auto.

## 9. INVESTIGACIÓN INTERNET

No se espera (todo local: scripts + manifest + ORT conocido). Sin red usada en esta tarea → sin citas. Si el runner hubiera necesitado descargar ORT/modelos, la fuente sería `github.com/microsoft/onnxruntime/releases` (release oficial, misma versión verificada en vivo por EMB-10). Estado: N/A + TSYS-13: ninguna cita URL presentada como evidencia.

## 10. VALIDACIÓN+CIERRE

- [ ] RED registrado (script inexistente → pwsh falla) ✅
- [ ] Slice 1 — NonInteractive + presencia + env + `--check` final ✅/⬜
- [ ] Slice 2 — interactivo (modelo/carpeta/descarga Q1+Q2, ollama/openai Q3 marcados) ✅/⬜
- [ ] Slice 3 — ORT ≥1.27 (detectar/cache-Temp/descargar + `ORT_DYLIB_PATH` sesión + doc en script) ✅/⬜
- [ ] `python embeddings/download.py --check` exit 0 ✅ (DISCOVERY; re-verificar al cerrar)
- [ ] Secrets audit: `grep Set-Content|Add-Content|Out-File.*key` = 0 writes + output maskeado (doubt-driven reconciliado)
- [ ] `git diff --check` limpio + alcance solo propios
- [ ] OCR (`pwsh dev-tools/ocr-review.ps1`) — advisory, Critical/High bloquean
- [ ] DoD 3 niveles (correctness+quality / integration+docs / ship-readiness con rollback=borrar .ps1)
- [ ] Reviewer P2-01: cambio no-crítico (script nuevo, sin prod code) → OCR + doubt-driven single-model; cross-model skipped (non-interactive)
- [ ] Gates D/V/C + RESULTADO §7 + Save Point
- [ ] Commit `feat:` SOLO propios (script + task file). Backlog→avance NO tocar. Push vía vanta-lead (este worker NO pushea).

## Steps

- [x] **Step 0 — DISCOVERY + task file:** contexto, archivos, Notion, gates, RED ✅ COMPLETO
- [x] **Step 1 — Slice 1 NonInteractive:** params `-NonInteractive/-Help`, defaults, presencia default, env sesión, `--check` final, mensaje éxito dim/idioma ✅ COMPLETO (exit 0)
- [x] **Step 2 — Slice 2 interactivo:** prompts Enter=auto, tabla Q2 con tamaño/tiempo+confirmación, qwen3 marcado GPU-only, Q3 ollama/openai marcados sin instalar ✅ COMPLETO (local/ollama/openai probados en vivo)
- [x] **Step 3 — Slice 3 ORT ≥1.27:** detectar (store/ENV/Temp-cache/System32) → copiar/descargar → `ORT_DYLIB_PATH` sesión + doc en header ✅ COMPLETO (copia offline Temp→store 16.4MB, rerun idempotente)
- [x] **Step 4 — cierre:** verify contrato + secrets audit + OCR + commit + recitation + RESULTADO ✅ COMPLETO

## Ejecución (evidencia mecánica)

### RED (2026-09-16, antes de crear el script)
- `pwsh -NoProfile -File setup-embeddings.ps1 -NonInteractive` → `El argumento "setup-embeddings.ps1" no se reconoce como el nombre de un archivo de script.` ✅ falla por razón correcta (archivo no existe).

### GREEN por slice
- **Slice 1:** `-NonInteractive` → provider=local, default presente, env sesión, `--check` OK, exit 0 ✅
- **Slice 2 (bugs systematic-debugging):** `$script:onnxOverride` bajo StrictMode (fix: init `$null`) + qwen3 listado como descargable (fix: flag GPU-only) + label de modo (fix: `(defaults, -NonInteractive)`→`($mode)`). Tras fix: NI exit 0 + interactivo Enter×3 exit 0 ✅
- **Slice 2 Q3 en vivo:** `ollama` → AVISO sin servidor + sigue sin instalar (exit 0); `openai` → `not set` + guía `$env:` sin pedir key (exit 0) ✅
- **Slice 3:** primera corrida copia Temp/ort130→`LOCALAPPDATA\VantaDB\onnxruntime\onnxruntime.dll` (16.4MB, v1.30 ≥1.27 ✓); rerun usa store (idempotente); `ORT_DYLIB_PATH` impreso solo-sesión ✅
- **`-Help`:** synopsis visible ✅. `git diff --check` exit 0 ✅. Alcance: solo `setup-embeddings.ps1` + este task file (EMB-12 paralela creó `vanta-mcp-local.ps1`+`EMB-12.md` — disjunta, NO tocada).

### Secrets audit (doubt-driven CLAIM: "el script nunca persiste ni loguea secrets")
- Adversarial: ¿parámetro de key? NO existe (diseño sin-parámetro). ¿write cmdlets? `Select-String 'Set-Content|Add-Content|Out-File'` = 0 matches. ¿key en output? solo `***set***`/`not set`. ¿Read-Host de key? prohibido por diseño (PSReadLine history→disco), documentado en `Test-OpenAiKey`. ¿URL con secreto? URL ORT fija pinneada, sin input de usuario.
- Reconciliación: 0 findings accionables (ruido: `Copy-Item`/`Expand-Archive` mueven solo DLL/zip ORT, no secrets). Veredicto: CLAIM SOSTIENE. Cross-model skipped (non-interactive, sin CLI externo autorizado) — anunciado.

### OCR delegation review (advisory)
- `ocr delegate rule setup-embeddings.ps1` → 1 grupo (default: correctness/security/performance/maintainability/tests). Self-review delegada: correctness verificada en vivo 6/6 paths; security sin superficie (URL fija, keys maskeadas, 0 writes); performance trivial; maintainability SLAP ok; coverage: paths de descarga real no ejercitados (todo presente offline — deuda menor, wrappers finos sobre `download.py` ya testeado en EMB-01).
- Veredicto: **sin Critical/High → no bloquea commit.**

### DoD 3 niveles
- Correctness+Quality: contrato 7/7 verificado en runtime; scope solo 2 archivos propios; sin código muerto.
- Integration+Docs: integra con `download.py --check`/`verify.py`/`config.rs` vars sin tocarlos; header del script = doc de UX + ORT; behavior timeless.
- Ship-readiness: secrets revisados; rollback = borrar el `.ps1` (aditivo puro); observabilidad = mensajes `[setup]` por paso; human review pendiente (orquestador/lead antes de push).

## Spec Gate

Scripts: N/A símbolos públicos Rust — solo UX texto (tabla §4). Gate D: blast radius 1 archivo nuevo + 0 ediciones, sin hot path, sin API pública, contrato mecánico + decisiones owner Q1-Q5 (Gate P del plan) → NO dispara `question`. Motivo: owner ya decidió + alcance aditivo mínimo.

## Context Save Point

DISCOVERY completo 2026-09-16. Todo presente en disco (3 modelos Q2 + ORT Temp cache). `download.py --check` OK. EMB-12 disjunta confirmada (sin `vanta-mcp-local.ps1`). Siguiente: Step 1 Slice 1 (esqueleto NonInteractive).
