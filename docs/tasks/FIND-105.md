# FIND-105: Comando único de instalación (one-liner por OS + wizard chain)

## Metadata

- **Plan file:** `docs/plans/2026-09-17-mvp-memoria-agentes.md` (Task 7, Wave3)
- **Fuente:** `docs/Backlog.md` línea FIND-105 ⬜ Pendiente + SPEC.md F1
- **Esfuerzo:** 🟢 1d (plan) · 2 steps downhill
- **Prioridad:** 🟠 Media-Alta
- **Tipo:** Docs/Distribución (scripts sh/ps1 + md; sin símbolos Rust públicos)
- **Turns estimados:** 5-10
- **Creado:** 2026-09-17
- **last-synced:** 2026-09-17
- **Estado:** ✅ COMPLETED (implementado + verificado; P2-01 y progreso los hace el orquestador)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0

## TAREA

- **Objetivo:** publicar el instalador como asset descargable sin clonar (raw URL versionada + checksum), con one-liner por OS (`irm|iex` / `curl|sh`) que encadena instalador→wizard interactivo (wizard FIND-104 ✅ `a1bea54b`), documentado en README/QUICKSTART. Incluye resto SHOW-05 (línea README + requirements ya ≥0.5.0).
- **Contrato:** install simulado/dry-run encadena wizard + docs con one-liner por OS + requirements verificado.
- **AC:** (a) install simulado/dry-run encadena wizard (ambos instaladores, exit 0, sin efectos); (b) docs con one-liner por OS (README + QUICKSTART, grep verificable); (c) requirements verificado (`vantadb-py>=0.5.0` en ambos requirements.txt, solo lectura).

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `README.md:203,209` + `README_ES.md:250,256` (one-liner citan los scripts) + `web/src/components/vanta/docs-view.tsx:263,270` (duplica one-liner) + `SPEC.md:36-38` (contrato) |
| Callees | GitHub Releases API (tag) + release assets (tarball/zip + `.sha256`) + raw.githubusercontent.com (wizard versionado) + `setup-embeddings.ps1` (wizard a encadenar, solo lectura) |
| Implicaciones | Sin símbolos Rust públicos; cambia comportamiento CLI de los instaladores (flags nuevos, default wizard ON con opt-out); docs EN (README/QUICKSTART); sin migración; sin perf/memoria |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `scripts/install.sh` (90L) · `scripts/install.ps1` (71L) · `README.md` (350L, foco `:192-218`) · `docs/QUICKSTART.md` (216L) · `setup-embeddings.ps1` (494L, wizard a encadenar) · `SPEC.md` (100L) · `.opencode/rules/release-ci.md` (42L) · `.opencode/references/definition-of-done.md` (144L) · `examples/demo/requirements.txt` (4L) · `benchmarks/requirements.txt` (33L) · `skills/vantadb-mcp/assets/install/` (6 plantillas, listado)
- **Archivos referenciados hacia dentro:** instaladores → `api.github.com/repos/ness-e/Vantadb/releases/latest` + `github.com/.../releases/download/<tag>/<asset>` + `<asset>.sha256`; wizard → `embeddings/manifest.json`, `vanta-mcp-local.ps1`, `skills/vantadb-mcp/assets/install/`
- **Archivos que referencian a los editados:** README.md, README_ES.md, docs-view.tsx, SPEC.md, Backlog FIND-105, plan Task 7 (ver grep § Investigation)
- **Veredicto impacto:** BAJO — scripts standalone sin callers de código; docs EN; riesgo = confianza `irm|iex` (mitiga: checksum in-script + URL oficial + nota manual) e idempotencia (mitiga: backup + re-ejecutable). RIESGO global: bajo.

## Contrato

`sh scripts/install.sh --dry-run` y `pwsh -NoProfile -File scripts/install.ps1 -DryRun` salen 0 mostrando la cadena instalador→wizard sin efectos; `Select-String` halla one-liner por OS en `README.md` y `docs/QUICKSTART.md`; ambos `requirements.txt` traen `vantadb-py>=0.5.0`.

## Spec (SDD — Phase 1b: sin símbolos públicos nuevos)

> Tabla Spec N/A para símbolos (distribución, sin `pub fn`/tools/endpoints). Decisiones de distribución resueltas por evidencia:

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Mecanismo de encadenamiento al wizard | A: instalador descarga wizard raw versionado y lo invoca (cero fricción; stdin pipe limita interactividad) / B: solo imprime next-step (robusto; fricción +1 paso) | A con fallback a B (intenta, si falla avisa sin romper install) | ✅ decidido-por-evidencia (SPEC.md:36-38 manda cadena; setup-embeddings.ps1:80-92 Read-Host funciona bajo `iex` con stdin=consola; bajo `curl\|sh` se documenta `--wizard-non-interactive`) |
| 2 | Dry-run | A: flag `--dry-run`/`-DryRun` que imprime sin efectos / B: nada (Stop del plan lo permite pero AC(a) exige simulado) | A | ✅ decidido-por-evidencia (contrato AC(a) + Stop "sin entorno limpio → dry-run") |
| 3 | Confianza `irm\|iex` | A: checksum in-script del tarball (ya existe) + nota verify manual / B: checksum del instalador (requiere asset nuevo = workflow = prohibido FIND-108) | A | ✅ decidido-por-evidencia (pre-mortem #1 pide checksum+URL oficial; workflows prohibidos) |
| 4 | Idempotencia | A: backup `.bak-<stamp>` del binario previo + `-Force` (re-ejecutable) / B: fallar si existe (fricción) | A | ✅ decidido-por-evidencia (pre-mortem #2; patrón Write-TextIfChanged setup-embeddings.ps1:232-246) |
| 5 | Docs | A: nota trust+cadena en README (ya trae one-liner `:192-210`) + §0 nueva en QUICKSTART (sin one-liner) / B: reescribir secciones | A | ✅ decidido-por-evidencia (README:192-210 existe; QUICKSTART:33-39 arranca con clone) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** WIP ajeno intacto (`.opencode`, `completions/*`, `docs/Backlog.md`, plan file salvo recitation, `reparacion.bat`, stash@{0} GOV-C4); `assets/install/` y `setup-embeddings.ps1` solo lectura; sin push; rama develop; commit `feat:` con staging selectivo (4 archivos + task file).
- **Comandos de verificación:** `sh scripts/install.sh --dry-run` (exit 0 + wizard) · `pwsh -NoProfile -File scripts/install.ps1 -DryRun` (exit 0 + wizard) · `Select-String "curl -fsSL|irm " README.md docs/QUICKSTART.md` · `Get-Content examples/demo/requirements.txt,benchmarks/requirements.txt` (floor ≥0.5.0) · `git diff --check` · `git status --short` (solo propios)
- **Deuda pendiente:** P2-01 review por agente distinto (lo hace el orquestador) + OCR delegation advisory + `skill progreso` (orquestador, Backlog/avance son suyos)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — 0 líneas Rust, sin dependencias nuevas, sin stubs. Wizard raw `main`-fallback documentado (no deuda: tag cuando hay release).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | AC(a)(b)(c) verificados mecánicamente (dry-run ×2 + grep ×2 + requirements ×2) |
| **Commit** | Commit atómico `feat: FIND-105 — ...`, solo 5 paths, `git diff --check` limpio, sin secrets |
| **Release** | N/A (distribución docs/scripts; no se publica crate; justificado) |

## Herramientas necesarias

- `pwsh -NoProfile` / `sh` dry-run + checksum verify + `campaign_verify_cmd` (bug exit -1 → bash directa)
- `Select-String` / `Get-Content` (verificación docs/requirements)
- codegraph N/A (scripts/docs, sin símbolos indexables — coverage `no_recorded_issue`, best-effort)

**Skills cargadas (SDP):** campaign-executor (base task-system) · progreso (base; ejecución la hace el orquestador) · ponytail-full (base; ladder mínimo-dif) · shipping-and-launch (flujo instalación + rollback `git revert`) · git-workflow-and-versioning (conventional `feat:`, staging selectivo, NO PUSH) · documentation-and-adrs (one-liner docs EN + changelog N/A justificado) · security-and-hardening (checksums, TLS, nunca secrets a disco, `irm|iex` con verify manual) · ci-cd-and-automation (no tocar workflows FIND-108; verify local). SDP v2 devolvió además lifecycle genéricas (incremental/test-driven/context/source/doubt/frontend/api-design): no cargadas — fuera del Top-8 justificado del plan (distribución sin lógica nueva;评分 plan manda). `SKILLS_CARGADAS: campaign-executor, progreso, ponytail-full, shipping-and-launch, git-workflow-and-versioning, documentation-and-adrs, security-and-hardening, ci-cd-and-automation`.

## Investigation Notes

- **Código (instaladores actuales):** tarball+sha256 ya implementados en ambos (`install.sh:52-77`, `install.ps1:26-56`); puntos de encadenamiento = tras `cp/Copy-Item` final (`install.sh:81-83`, `install.ps1:60`) — ahí va el bloque wizard + resumen. `install.ps1` ya tolera fallo API con fallback `v0.4.0` (`:22`) — se reutiliza el patrón para el wizard (warn sin romper).
- **Problema (confianza + idempotencia):** `irm|iex`/`curl|sh` solo aceptables con TLS (`--ssl-reqd`/`Tls12` ya) + host oficial + checksum in-script del payload (ya) + nota de verificación manual documentada (nuevo). Idempotencia: `mkdir -p`/`-Force` ya; falta backup del binario previo (nuevo, patrón `Write-TextIfChanged` backup fechado).
- **Internet:** patrones Deno/Bun ya researched (reusar, sin nueva búsqueda): `curl -fsSL <url> | sh` + `irm <url> | iex`, flags `--no-*`, checksum publicado junto al asset. Sin gaps → sin web research (evita ruido; SPEC.md:30 ya fija restricción "descargas versionadas con checksum").
- **Resto SHOW-05:** requirements ambos en `>=0.5.0` ✅ (demo:1, benchmarks:12); fila README→demo/colab ya existe (README:65, commit FIND-74 `5e428aea`); línea QUICKSTART→examples ya existe (QUICKSTART:216). Resto = solo verificación (c), sin ediciones en requirements.
- **Colaterales NO tocados (scope):** `README_ES.md:250,256` y `web/src/components/vanta/docs-view.tsx:263,270` duplican el one-liner viejo (sin cadena wizard) — archivos fuera de Archivos clave → Gate C candidato FIND futuro, no inline.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% (2/2 steps ✅ + verify contrato ✅; P2-01 + progreso = orquestador) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — trust boundary = descarga+ejecución remota (`curl|sh`, `irm|iex`). Skill `security-and-hardening` aplicada: TLS obligatorio (existente), host oficial pineado, checksum sha256 del payload verificado in-script, secrets N/A (instalador no maneja keys; wizard las lee solo de env — lectura), verify manual documentado, `git diff --staged | grep -i secret` pre-commit. Sin dependencias nuevas → `cargo audit/deny` N/A justificado.
- [x] **PERFORMANCE** — N/A justificado: scripts de instalación (1 ejecución, red-dominado); sin hot paths, sin serialización, sin loops calientes.

## Steps

### Step 1: Instaladores encadenan wizard (dry-run + backup + opt-out)

- **Archivos:** `scripts/install.sh`, `scripts/install.ps1`
- **Acción:** flags `--dry-run/--no-wizard/--wizard-non-interactive` (sh) y `-DryRun/-NoWizard/-WizardNonInteractive` (ps1); backup `.bak-<stamp>` del binario previo; bloque wizard post-install (local `setup-embeddings.ps1` si hay checkout, si no raw versionado `<tag>` con fallback `main`; warn sin romper); resumen final con next-step. ~100 líneas entre ambos.
- **Verify:** `sh scripts/install.sh --dry-run` exit 0 + menciona wizard ✅ (Git sh, 2026-09-17); `pwsh -NoProfile -File scripts/install.ps1 -DryRun` exit 0 + menciona wizard ✅; `sh -n` syntax ✅; `--no-wizard`/`--help` variants ✅
- **Estado:** ✅ COMPLETED

### Step 2: Docs con one-liner por OS + cadena wizard

- **Archivos:** `README.md`, `docs/QUICKSTART.md`
- **Acción:** README § One-Line Installation: nota trust (TLS+sha256+verify manual) + línea cadena wizard (opt-out). QUICKSTART: nueva §0 "Install without cloning" (one-liner por OS + cadena wizard + nota requirements `vantadb-py>=0.5.0`). EN ambos. ~30 líneas.
- **Verify:** `Select-String` one-liner en ambos ✅ (README:203,209 + QUICKSTART:28,34,42,47) + `git diff --check` ✅ + `validate-docs-coverage.ps1` 0 gaps ✅
- **Estado:** ✅ COMPLETED

## Dependencias

- FIND-104 ✅ `a1bea54b` (wizard a encadenar — debe completarse antes) · Wave3 paralela FIND-108 (disjuntos: scripts/docs vs workflows; no tocar `.github/workflows/`, `.opencodereview/`)
- NextTask: ninguna (última wave; el orquestador hace cierre + progreso)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** orquestador (vanta-review / agente distinto — NO este contexto)
- **Enfoque:** ¿cadena installer→wizard honesta sin clone? ¿dry-run sin efectos? ¿scope (5 paths)?
- **Cómo se probó:** dry-run ×2 exit 0 + grep docs + requirements + `git diff --check` (evidencia abajo)
- **Checklist anti-hábitos tóxicos:** ver cierre
- **Veredicto:** ⏳ pendiente orquestador

## Notas

- **Rama `develop` confirmada; WIP ajeno mapeado e intacto** (`.opencode`, `completions/*`, `docs/Backlog.md`, plan file, `reparacion.bat` — NO stageados).
- **Evidencia verify (2026-09-17):** sh `--dry-run`/`--no-wizard`/`--help` exit 0 (wizard chain impresa); ps1 `-DryRun` exit 0; requirements `vantadb-py>=0.5.0` ×2; `git diff --check` 0; secrets-scan 0; `validate-docs-coverage.ps1` 0 gaps; `ocr delegate rule` 4/4 paths → 1 grupo default (advisory, sin Critical/High; input para P2-01). `campaign_verify_cmd` bug exit -1 reproducido (riesgo conocido del plan) → fallback bash directa documentado.
- **Checklist anti-hábitos tóxicos (self-check pre-P2-01):** salidas todas ejecutadas de verdad (logs arriba); sin clarificación omitida (contrato del plan); done solo tras AC(a)(b)(c) mecánicos; sin fallos ignorados (exit -1 reportado, no oculto); búsqueda en 2 niveles (grep repo + lectura directa); citas con file:línea; sin reintentos en bucle (1 fix dirigido: dry-run pre-OS); cada step al contrato; sin degradación de errores (wizard falla → warn, install intacto); paradas explícitas (Stop dry-run respetado, sin red real).
- Rollback: `git revert <commit>` (distribución docs/scripts, sin migración; shipping-and-launch §2b).
