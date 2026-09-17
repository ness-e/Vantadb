# FIND-108: integración completa open-code-review

## Metadata
- **Plan file:** docs/plans/2026-09-17-mvp-memoria-agentes.md
- **Fuente:** plan file Task 8 (Wave3) + research OCR 2026-09-17
- **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟡 Media
- **Tipo:** DevOps/CI (config, no código Rust)
- **Turns estimados:** 15-30
- **Creado:** 2026-09-17
- **last-synced:** 2026-09-17
- **Estado:** ✅ COMPLETO-técnico (pendiente P2-01 orquestador)
- **Incógnitas (uphill):** 0 (approach validado: rule.json + nightly delegation-default + viewer-evidencia)
- **Pendientes (downhill):** 0 (3/3 steps ✅ + DISCOVERY)
- **Branch:** develop · **Commit:** `ci: FIND-108 — ...` · **NO PUSH**

## 1. TAREA — objetivo + contrato + AC

**Objetivo:** completar la integración open-code-review (`ocr` v1.12.5 instalado vía npm) más allá del delegation ya integrado (`dev-tools/ocr-review.ps1` + gates; `.opencode/references/ocr-review.md`). Hoy SIN integrar: review-rules por path, job CI, plugins, skill portable, MCP server, session viewer, telemetría (research 2026-09-17, fuente README upstream + webfetch). Esta tarea integra lo que es viable sin key ni dependencias externas: **rules por path + job CI nocturno + session viewer como evidencia**. Plugins/skill-portable/MCP-server/telemetría quedan explícitamente FUERA (requieren endpoint LLM / distribución; documentado en Notas, no deuda silenciosa).

**Contrato:** rules mapeadas a `.opencode/rules/` + job CI nocturno verde (o documentado-sin-key) + viewer como evidencia + delegation intacto.

**AC (criterios de aceptación):**
- (a) **rules por path mapeadas 1:1** a reglas VantaDB: `.opencodereview/rule.json` existe, con una entrada por cada archivo de `.opencode/rules/` (13 reglas → 13 entradas), texto condensado fiel al Must/Must-not original, `merge_system_rule: true`. Verificación mecánica: `ocr rules check --rule .opencodereview/rule.json <path-representativo>` resuelve regla custom por área.
- (b) **job CI nocturno verde o documentado-sin-key:** `.github/workflows/ocr-nightly.yml` NUEVO (workflows existentes SOLO lectura). Job `delegate` corre siempre sin key (exit 0); job `full` corre `ocr review/scan` solo si `secrets.OCR_LLM_TOKEN` existe, si no hace skip documentado en verde. Delegation = default sin key; full solo con key. Validación: `actionlint` exit 0.
- (c) **session viewer como evidencia de auditoría:** el job nocturno sube artefactos (`ocr-preview.json`, `ocr-rules.json`, `ocr-result.json` cuando hay key) y el header del workflow + este task file documentan el replay local (`ocr session list/show`, `ocr viewer`). Evidencia = artefactos versionados por run, replay determinista local.
- (d) **delegation intacto:** `dev-tools/ocr-review.ps1` SIN cambios (el resolver OCR carga `.opencodereview/rule.json` del repo automáticamente a alta prioridad — probado: `ocr delegate rule` ya agrupa por contenido; las rules custom se suman). Self-check: `pwsh dev-tools/ocr-review.ps1 -Format json` sigue exit 0 y ahora resuelve grupos con reglas VantaDB.

## 2. ARCHIVOS — clave + relacionados + prohibidos

**Clave:**
- `.github/workflows/` → **UN archivo job NUEVO** `ocr-nightly.yml` (workflows existentes SOLO lectura, ni un byte).
- `.opencodereview/` → **dir NUEVO** con `rule.json` (rules por path, formato upstream `{"rules":[{"path","rule","merge_system_rule"}]}`).
- `dev-tools/ocr-review.ps1` → **lectura** (78 líneas leídas completas); extensión mínima solo si hace falta, sin romper delegation. Veredicto: NO se toca (el resolver auto-carga rule.json).

**Relacionados (lectura):**
- `.opencode/rules/` (13 archivos — origen del mapeo 1:1; `release-ci.md` leído completo, resto por headings + `durability.md` completo).
- `.opencode/references/ocr-review.md` (gates), `definition-of-done.md` (DoD 3 niveles).
- Research OCR 2026-09-17 (README upstream vía webfetch + `ocr --help` en vivo v1.12.5).

**PROHIBIDOS (ni leer-para-editar ni stagear):**
`scripts/install.*`, `README.md`, `docs/QUICKSTART.md` (FIND-105 en paralelo — NO tocar), `reparacion.bat`, `.opencode` vivo (edits — las rules van a `.opencodereview/`, nunca a `.opencode/`), `Justfile`, `completions/*`, `desktop/src-tauri/Cargo.lock`, `stash@{0}` GOV-C4, `docs/Backlog.md`, plan file (solo append recitation al cierre, nunca re-spec), `src/`, `examples/`, `vanta-memory/`, `vantadb-mcp/`, `skills/`, `setup-embeddings.ps1`, `vanta-mcp-local.ps1`. WIP ajeno visible en `git status` (`completions/*`, `.opencode`, `docs/Backlog.md`, plan) → staging SELECTIVO (`git add` solo paths de esta tarea).

## 3. DEPENDENCIAS

- **Wave3** (FIND-105 ✅/paralelo, disjuntos: `scripts/install.*`+docs vs workflows+rules — cero solape).
- **No bloquea a nadie.** **NextTask: ninguna** (última wave; cierre del orquestador).
- **Stop:** sin key disponible → delegation + rules locales (no bloquea). El job `full` queda documentado, no activo sin secret.

## 4. REFERENCIAS

- **Rules** `.opencode/rules/release-ci.md` — lectura COMPLETA (reglas 1-5: allocator, sccache, MSRV/COPYs, version sync, `continue-on-error` + CATEGORY tag). Resto de rules: headings + condensado fiel en `rule.json`.
- **Refs:** `ocr-review.md` (gates VERIFY, wrapper, CI INFORMATIONAL), `dev-tools.md`, `definition-of-done.md` (DoD 3 niveles + capa determinista + progreso checklist).
- **Commands:** `pipeline.md` (ejecución), `audit.md` (verify L9/post-tarea).
- **Docs open-codereview.ai** (rules/cicd/delegate/mcp) vía webfetch → URLs `open-codereview.ai/docs/*` dieron 404 (sitio reestructurado); fuente efectiva: README upstream `github.com/alibaba/open-code-review` + `action.yml` (raw) + `ocr --help` en vivo. Tabla Spec: N/A (CI/config, sin símbolos públicos nuevos → no es feature-add para SDD, sin `## Spec`).

## 5. SKILLS — SDP real (Paso 0b), ≤8

`campaign_discover_skills_v2` (phase BUILD, 14 keywords) devolvió 8 genéricas lifecycle (campaign-executor, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, source-driven-development, frontend-ui-engineering, api-and-interface-design). Ajuste lead: quedarse con las que aplican a CI/config + sugeridas del plan, descartar UI/API (frontend-ui-engineering, api-and-interface-design, incremental-implementation, test-driven-development, context-engineering NO aplican — config YAML/JSON sin lógica nueva).

**SKILLS_CARGADAS:** ci-cd-and-automation (job CI nocturno, gates, CATEGORY tags) · documentation-and-adrs (rule.json como doc viva + header workflow) · git-workflow-and-versioning (commit `ci:`, staging selectivo, sin push) · systematic-debugging (Iron Law si verify falla) · source-driven-development (docs OCR verificadas contra README upstream + CLI en vivo, no memoria del modelo) + base (campaign-executor/progreso/ponytail auto).
**SDP:** discover-v2 + filtrado lead justificado (5/8, resto descartado por no-aplicar).

## 6. HERRAMIENTAS + MCP

- `ocr delegate rule` (sin key — default) + `ocr rules check --rule` (verificar mapeo) + `ocr session list/show` + `ocr viewer` (evidencia local).
- `actionlint` (validar workflow nuevo — instalado `C:\Users\Eros\AppData\Local\Microsoft\WinGet\...`, verificado).
- agent-search/metasearchmcp → usé webfetch directo (README + action.yml raw); metasearch innecesario (fuente primaria resolvió).
- `campaign_verify_cmd` (bug exit -1 conocido → fallback bash directa).
- **Secrets/keys NUNCA a disco ni al repo** (el job full con key queda documentado con `if: secrets.OCR_LLM_TOKEN != ''`, no activo sin secret).
- codegraph N/A (config YAML/JSON, no código — sin blast radius de símbolos).

## 7. INVESTIGACIÓN CÓDIGO (DISCOVERY — hecha)

- **Wrapper actual** (`dev-tools/ocr-review.ps1`, 78L completo): `ocr delegate preview --format json` + `ocr delegate rule <files> --format json` → spec `{schema: ocr-delegate/v1, preview, rules}`; excluye `.opencode`; exit 0 advisory. NO necesita cambios: el resolver OCR lee `<repo>/.opencodereview/rule.json` a alta prioridad automáticamente (evidencia: `action.yml` upstream, step fingerprint `localRuleDigest`).
- **Workflows existentes como patrón** (solo lectura): `ocr-delegate.yml` (71L — INFORMATIONAL, todo `continue-on-error: true` + `# CATEGORY:`, artefactos `ocr-preview.json`+`ocr-rules.json`) y `heavy-bench-nightly-51.yml` (patrón `schedule: cron '0 3 * * *'` + `workflow_dispatch` + artefactos con retention). El nightly nuevo compone ambos patrones.
- **Rules a mapear:** 13 archivos `.opencode/rules/` (headings extraídos por grep; `release-ci.md` y `durability.md` completos). Mapeo path→regla en Step 1.

## 8. INVESTIGACIÓN PROBLEMA

- **Ruido (pre-mortem 2):** rules genéricas ruidosas → mitigación = mapeo 1:1 a reglas VantaDB (cada entrada cita Must/Must-not concretos del repo, no consejos genéricos). `merge_system_rule: true` conserva además la system rule built-in (no se pierde cobertura `**/*.rs`→Regla 4).
- **Key ausente (pre-mortem 1 + Stop):** delegation documentado como default (job `delegate` siempre verde sin key); full nocturno (`ocr review --format json` + artefactos + sessions) solo con `secrets.OCR_LLM_TOKEN`. Sin secret el job hace skip con mensaje — workflow verde, documentado, no bloquea.

## 9. INVESTIGACIÓN INTERNET (digest ≤500 palabras + URLs verificadas; research 2026-09-17 como base)

**Fuentes verificadas (webfetch 2026-09-17, todas resolvieron salvo docs-site):**
1. `https://github.com/alibaba/open-code-review` — README: delegation mode = scaffolding determinista + host LLM, sin key; `ocr delegate preview/rule`; `ocr review/scan --format json`; plugins por agente (Claude/Codex/Cursor/OpenCode); `action.yml` para CI con key; session viewer; telemetría OTel. 34.3k stars, `ocr` v1.12.5 en vivo coincide.
2. `https://raw.githubusercontent.com/alibaba/open-code-review/main/action.yml` — action `OpenCodeReview PR Review`: inputs `llm_url/llm_auth_token/llm_model` (requeridos), `rule` (custom rules path), artefactos, sticky/incremental. Requiere key → solo para job `full` con secret.
3. `https://github.com/alibaba/open-code-review/tree/main/.opencodereview` — contiene SOLO `rule.json` (9 líneas, 1.61 KB): formato `{"rules":[{"path":"internal/llm/providers.go","rule":"...","merge_system_rule":true}]}`. Es el formato canónico a replicar.
4. CLI en vivo (`ocr --help`, `ocr delegate/session/viewer/scan --help`): `rules check <path> [--rule]` (qué regla aplica por path + capa origen); `session list/show/comments/compare` (sessions en `~/.opencodereview/sessions/`); `viewer` (WebUI local `:5483`); `scan --path --preview` (full-file sin diff).
5. NO verificadas (404): `https://open-codereview.ai/docs[/review-rules|/cicd|/delegate|/viewer]` — sitio reestructurado; NO citar como fuente. [citas NO VERIFICADAS — docs-site pendiente re-chequeo]
**Decisión derivada:** rules por path = `rule.json` (formato upstream probado); CI nocturno = componer `ocr-delegate.yml` (sin key) + `action.yml` (con key, solo `full`); viewer = evidencia local sobre artefactos del nightly (el viewer es WebUI interactiva, no corre en CI — el artefacto JSON es la evidencia versionada, el viewer el replay).

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `dev-tools/ocr-review.ps1` (lee rule.json vía resolver, sin cambio) · `.github/workflows/ocr-nightly.yml` (nuevo, standalone) · agentes que consumen `ocr delegate rule` (reciben grupos + rules VantaDB) |
| Callees | `ocr` CLI v1.12.5 (npm global) · `.opencode/rules/*.md` (solo lectura como fuente) · `actions/checkout@v4`, `actions/setup-node@v4`, `actions/upload-artifact@v4` (pins del patrón existente) |
| Implicaciones | contrato público: ninguno (sin símbolos `pub`, sin endpoints, sin bindings) · comportamiento: `ocr delegate rule` devuelve grupos con reglas VantaDB además de system (aditivo) · performance: N/A (CI nocturno, fuera del Fast Gate) · migración: ninguna · tests existentes: ninguno afectado (sin código) |

**RIESGO: bajo.** Archivos nuevos salvo wrapper intacto; CI nocturno no toca Fast Gate; sin secrets.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `dev-tools/ocr-review.ps1` (78L) · `.opencode/rules/release-ci.md` (42L) · `.opencode/rules/durability.md` (34L) · `.opencode/references/ocr-review.md` (65L) · `.opencode/rules/README.md` (111L) · `.github/workflows/ocr-delegate.yml` (71L) · `.github/workflows/heavy-bench-nightly-51.yml` (334L, patrón schedule/artefactos).
- **Archivos referenciados hacia dentro:** el workflow nuevo referencia `actions/checkout@v4`, `actions/setup-node@v4`, `actions/upload-artifact@v4`, `ocr` (npm), `.opencodereview/rule.json`; rule.json no referencia archivos (texto autocontenido).
- **Archivos que referencian a los editados:** `dev-tools/ocr-review.ps1` NO referencia rule.json por path (resolver implícito — verificado en action.yml: `localRuleDigest`); `.opencode/references/ocr-review.md` menciona `ocr-delegate.yml` (no se edita); ningún workflow existente referencia `ocr-nightly.yml` (standalone).
- **Veredicto impacto:** BAJO — 2 paths nuevos, 0 edits. Si rule.json es inválido, `ocr delegate rule` lo ignora con warning (no rompe delegation); si el workflow falla, es schedule aislado (no bloquea PRs). Reversión: `git rm` de los 2 paths.

## Contrato

"`ocr rules check --rule .opencodereview/rule.json` resuelve regla VantaDB por área + `actionlint .github/workflows/ocr-nightly.yml` exit 0 + `pwsh dev-tools/ocr-review.ps1 -Format json` exit 0 con grupos + delegation sin cambios"

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** delegation default sin key (wrapper exit 0 sin config LLM) · `.opencode/` vivo intacto (cero edits) · WIP ajeno intacto (staging selectivo) · sin secrets en repo · Fast Gate <5min intacto (nightly fuera del critical path) · Regla 2: ningún `continue-on-error` nuevo sin `# CATEGORY:`.
- **Comandos de verificación:** `ocr rules check --rule .opencodereview/rule.json <path>` · `actionlint .github/workflows/ocr-nightly.yml` · `pwsh dev-tools/ocr-review.ps1 -Format json` · `git status --short` (solo 2 paths nuevos + este task file).
- **Deuda pendiente:** P2-01 review por agente distinto (orquestador) · plugins/skill-portable/MCP-server/telemetría OCR fuera de scope (decisión, no deuda).

## Recitation (canónico — estructura única)

*(se sincroniza al cerrar vía plan-file recitation + RESULTADO §7 pipeline-full)*

## Deuda técnica (Regla 6 — MUST)

**Sin deuda.** Saldo neto 0: no se introduce deuda nueva (config aditiva, sin shortcuts, sin stubs). Plugins/skill/MCP/telemetría = scope explícitamente excluido con justificación (requieren endpoint LLM), no deuda silenciosa.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | AC (a)-(d) cumplen + `ocr rules check` por área OK + actionlint OK + wrapper self-check OK |
| **Commit** | commit atómico `ci: FIND-108 — ...`, solo 3 paths (rule.json + workflow + task file), `git diff --check` limpio |
| **Release** | N/A (no user-visible, no changelog; verify.ps1 completo lo corre el orquestador al cierre Wave3) |

## Herramientas necesarias

- `ocr` CLI v1.12.5 · `actionlint` · `pwsh` · `git` · webfetch (hecho en DISCOVERY)

**Skills cargadas (SDP):** ci-cd-and-automation (job nocturno + CATEGORY tags) · documentation-and-adrs (rule.json + header workflow como doc viva) · git-workflow-and-versioning (commit `ci:`, staging selectivo, NO PUSH) · systematic-debugging (si verify falla, Iron Law) · source-driven-development (README upstream + CLI en vivo como fuente, no memoria) + base campaign-executor/progreso/ponytail.

## Investigation Notes

- Ver §9 (digest + URLs). Hallazgo clave: `.opencodereview/` upstream = SOLO `rule.json` con `merge_system_rule` — replicar exacto.
- `ocr rules check` sin `--rule` usa embedded + repo rule.json (auto-descubrimiento confirmado por `localRuleDigest` en action.yml).
- `open-codereview.ai/docs/*` 404 ×5 → no citar; re-chequear en otro momento (anotado como cita-no-verificada, no como evidencia).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 (S1 rule.json ✅ · S2 workflow ✅ · S3 verify+commit ✅) |
| % completado | 100% (DISCOVERY + 3/3 steps verificados + commit; pendiente P2-01 orquestador) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluado: el cambio toca CI (trust boundary: secrets). Mitigación: job `full` usa `secrets.OCR_LLM_TOKEN` solo vía env (`OCR_LLM_TOKEN`), nunca echo/log; `permissions: contents: read` mínimo; sin escritura en repo; `security-and-hardening` NO cargada (sin input de usuario ni código — justificado).
- [x] **PERFORMANCE** — N/A justificado: config estática + CI nocturno fuera de hot paths (sin loops, sin serialización, sin benchmarks). Sin baseline que medir.

## Steps

### Step 1: `.opencodereview/rule.json` — rules por path 1:1
- **Archivos:** `.opencodereview/rule.json` (NUEVO, ~13 entradas)
- **Acción:** escribir rule.json formato upstream; cada entrada `path` (glob del área) + `rule` (condensado fiel Must/Must-not de su `.opencode/rules/*.md`) + `merge_system_rule: true`
- **Verify:** `ocr rules check --rule .opencodereview/rule.json <13 paths representativos>` → cada uno resuelve su regla custom
- **Estado:** ✅ (2026-09-17: 13/13 `Source: Custom (--rule)` + Pattern correcto + exit 0 c/u; sin ajustes de globs)

### Step 2: `.github/workflows/ocr-nightly.yml` — job CI nocturno
- **Archivos:** `.github/workflows/ocr-nightly.yml` (NUEVO)
- **Acción:** job `delegate` (siempre, sin key: preview+rules+artefactos) + job `full` (`if: secrets.OCR_LLM_TOKEN != ''`: review json + artefactos; sin secret: skip documentado verde). Header comenta viewer como replay. Pins como `ocr-delegate.yml`; CATEGORY tags en cada `continue-on-error`.
- **Verify:** `actionlint .github/workflows/ocr-nightly.yml` exit 0
- **Estado:** ✅ (2026-09-17: actionlint 1.7.12 exit 0 tras agregar job `check-key` faltante — el draft referenciaba `needs: [check-key]` sin definirlo; fix mínimo: job que exporta `has_key` desde `secrets.OCR_LLM_TOKEN` sin exponer el secret)

### Step 3: verify contrato + self-check + commit
- **Archivos:** `docs/tasks/FIND-108.md` (sync), plan file (append recitation)
- **Acción:** OCR self-check (`ocr-review.ps1 -Format json` exit 0 + grupos con reglas VantaDB) → DoD 3 niveles → `git add` SELECTIVO (solo 3 paths) → commit `ci:` (NO PUSH)
- **Verify:** contrato completo §Contrato + `git status --short` limpio de ajenos + `git diff --check`
- **Estado:** ✅ (2026-09-17: wrapper exit 0, spec `ocr-delegate/v1`, 6 archivos → 3 grupos; `ocr-nightly.yml` agrupa `source: project` + release-ci custom; `git diff --check` exit 0; staging selectivo 3 paths; commit `ci:`, NO PUSH)

## Dependencias

- FIND-105 (Wave3 paralelo, disjunto) — sin dependencia técnica
- NextTask: ninguna (última wave; cierre del orquestador)

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta el ORQUESTADOR (agente distinto al implementador). Sin esto registrado, la tarea no está COMPLETED para el plan (queda COMPLETO-técnico pendiente de P2-01).

- **Revisor:** orquestador (vanta-review / vanta-lead pipeline)
- **Enfoque:** ¿mapeo 1:1 fiel? ¿job verde-sin-key correcto? ¿delegation intacto?
- **Cómo se probó:** `ocr rules check` ×13 + actionlint + wrapper self-check (outputs abajo en Notas al cerrar)
- **Checklist anti-hábitos tóxicos:** pendiente al cierre
- **Veredicto:** ⬜ pendiente (orquestador)

## Notas

- Gate D: NO disparado (blast radius 2 paths nuevos, sin hot path/API pública/símbolos nuevos, contrato no ambiguo).
- Gate P: ya confirmado a nivel plan (Q5 2026-09-17).
- (resultados de verify + outputs se anexan al cerrar cada step)
- **Cierre 2026-09-17 (reanudación tras abort):**
  - S1: `ocr rules check --rule .opencodereview/rule.json` ×13 → todos `Source: Custom (--rule)`, exit 0 (wal→durability, hnsw→indexes, planner→query-dsl, memory_governor→memory-budget, ingestion→concurrency, engine→core-engine, sdk→api-contract, server→server-mcp, python→python-bindings, wasm→js-ecosystem, page.tsx→frontend-web, Cargo.toml→open-core-licensing, Dockerfile→release-ci). Self-test del workflow (`sed -n '2p'` + `grep -q Custom`) válido contra este formato.
  - S2: `actionlint` 1.7.12 → 2 errores `job-needs` por `check-key` no definido (draft); fix: job `check-key` (outputs.has_key desde secret, secret solo en `run:`, nunca a disco) → exit 0. Pins espejados de `ocr-delegate.yml` (checkout/setup-node/upload-artifact @v4). Schedule 04:00 UTC sin colisión con heavy-bench 03:00. `continue-on-error` único con `# CATEGORY: BEST-EFFORT` (Regla 2 OK).
  - S3: `pwsh dev-tools/ocr-review.ps1 -Format json` → exit 0, spec `ocr-delegate/v1`, 6 archivos → 3 grupos; `ocr-nightly.yml` → `source: project`, pattern release-ci + User-Specific Rules VantaDB (merge_system_rule funciona end-to-end). `.opencodereview/rule.json` cae en regla genérica JSON (sin glob propio — esperado, fuera del mapeo 1:1). Wrapper SIN cambios (delegation intacto). Nota: invocar con `pwsh` (7.x); `powershell` 5.1 falla parseando el wrapper (preexistente, no de esta tarea).
  - Intel orquestador corregida: `.github/workflows/ocr-delegate.yml` está TRACKED e intacto (no `??`); WIP ajeno real: `completions/*`, `.opencode`, `docs/Backlog.md`, plan file, `reparacion.bat` — ninguno stageado.
