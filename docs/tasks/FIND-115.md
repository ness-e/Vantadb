# FIND-115 — sync one-liner viejo (README_ES + docs-view) con FIND-105

> **Plan:** `docs/plans/2026-09-17-seguimiento-mvp.md` (Wave2, última del plan)
> **Campaign ID:** 64985e0b-0570-431c-a1e9-1d0d551ad54e
> **Estado:** ✅ COMPLETED (commit 39000b2c)
> **Branch:** develop · **Commit:** `docs: FIND-115 — ...` · **Appetite:** max 1h · **Esfuerzo:** 🟢
> **SDP:** documentation-and-adrs (+ base campaign-executor/progreso/ponytail; lifecycle BUILD sugería frontend-ui-engineering pero el cambio web es 1 párrafo de texto, sin componentes — se descarta con motivo)

## 1. TAREA — objetivo + contrato + AC

**Objetivo:** sincronizar las superficies con instrucciones de instalación viejas con el one-liner vigente FIND-105 (`1349e63c`), o referenciar la fuente única.

**Contexto:** tres superficies mostraban instrucciones distintas → rompe confianza en instalación. Fuente única = one-liner FIND-105 (`1349e63c`: `README.md` § One-Line Installation + `docs/QUICKSTART.md` §0), que añade sobre el comando bare: bloque `Trust:` (TLS + verificación `.sha256`), cadena al wizard (`--no-wizard`/`-NoWizard`), y preview sin efectos (`--dry-run`/`-DryRun`).

**Hallazgo DISCOVERY (2026-09-18):** los comandos bare son byte-idénticos en las 4 superficies (`curl -fsSL .../install.sh | sh`, `irm .../install.ps1 | iex`). Lo "viejo" es la ausencia del contexto FIND-105 (Trust/wizard/dry-run) en README_ES y docs-view. Path web confirmado: `web/src/components/vanta/docs-view.tsx:263,270` — NO renombrado, NO hay SKIP parcial.

**Contrato:** las superficies existentes muestran el one-liner FIND-105 o referencian a la fuente única + `validate-docs-coverage.ps1` 0 gaps.

**AC:**
- [ ] README_ES muestra el one-liner vigente o referencia a la fuente única (sin re-traducción)
- [ ] docs-view muestra el one-liner vigente o referencia a la fuente única
- [ ] `scripts/validate-docs-coverage.ps1` → 0 gaps

## 2. ARCHIVOS — clave + relacionados + prohibidos

**Clave (editar):**
- `README_ES.md:250,256` — comandos OK, falta contexto FIND-105 → REFERENCIAR (nota ES breve a fuente única)
- `web/src/components/vanta/docs-view.tsx:263,270` (path VERIFICADO en DISCOVERY vía glob — existe, 666L) — comandos OK, falta contexto → UNIFICAR mínimo (párrafo Trust+wizard+dry-run EN + referencia)

**Relacionados (lectura):**
- `README.md:192-221` (fuente única, lectura)
- `docs/QUICKSTART.md:19-53` §0 (fuente única, lectura)

**Prohibidos (no tocar):**
re-traducción ES (solo el one-liner/nota); resto de `web/`; `scripts/install.*` (FIND-105 ✅); resto del repo; `reparacion.bat`; `.opencode`; `Justfile`; `ocr-*`; `completions/*`; `desktop/src-tauri/Cargo.lock`; stash@{0} GOV-C4; `docs/Backlog.md`; plan file (solo recitation); `C:/Users/Eros/.vantadb*`; `SPEC.md:36-37` (placeholders `<release-url>` = spec futura, scope creep — NOTICED BUT NOT TOUCHING); `embeddings/README.md:66` (mención de índice, no superficie de instalación).

## 3. DEPENDENCIAS — wave, stop, next

- **Wave2 última en secuencia** (FIND-102 ✅, FIND-114 ✅ verificados en recitations del plan).
- **Stop:** superficie inexistente → SKIP parcial con evidencia (no cazar fantasmas). NO disparado: ambas superficies existen.
- **NextTask:** ninguna (última del plan; cierre del orquestador).

## 4. REFERENCIAS

- **Agents:** `vanta-worker` (implementa) + `vanta-review` P2-01 (orquestador).
- **Refs:** `.opencode/references/definition-of-done.md`.
- **Commands:** `.opencode/commands/pipeline.md`.
- **SPEC.md raíz:** N/A (docs; SPEC.md:36-37 con placeholders queda fuera — ver §2).
- **Tabla Spec:** N/A (docs, no feature-add — sin símbolos públicos nuevos; Gate mecánico spec-first no aplica).

## 5. SKILLS

- **Base en sesión:** campaign-executor, progreso, ponytail(full).
- **Sugerida plan:** documentation-and-adrs ✅ cargada.
- **SDP real (Paso 0b):** `campaign_discover_skills_v2` → 8 (campaign-executor, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, frontend-ui-engineering, api-and-interface-design). De las 8 solo se adopta documentation-and-adrs (keyword-mapped, la que cubre el dominio docs); frontend-ui-engineering se descarta con motivo (cambio web = 1 párrafo de texto, sin componentes/estilos); TDD/incremental no aportan (sin lógica ni tests); doubt-driven no aplica (sin trust boundary de código).
- **SKILLS_CARGADAS:** documentation-and-adrs

## 6. HERRAMIENTAS + MCP

- `scripts/validate-docs-coverage.ps1` (contrato: 0 gaps)
- `campaign_verify_cmd` — bug conocido exit -1 → fallback bash directa (riesgo global del plan)
- codegraph: N/A (docs, sin símbolos)
- Internet: N/A

## 7. INVESTIGACIÓN CÓDIGO

N/A código. DISCOVERY = ubicar superficies reales:
- glob `web/**/*docs-view*` → `web/src/components/vanta/docs-view.tsx` ✅ (path del plan correcto, 666L, InstallCard "Precompiled CLI binary" :256-280)
- grep `install\.sh|install\.ps1` repo → 4 superficies con comandos: README.md:203,209 / QUICKSTART:28,34 (+ dry-run :42-48) / README_ES:250,256 / docs-view:263,270. Comandos idénticos; delta = contexto FIND-105.
- `git show 1349e63c --stat` → 5 files (+325/-3), README_ES no tocado → confirma drift.

## 8. INVESTIGACIÓN PROBLEMA — fuente única vs duplicación (decisión por superficie)

| Superficie | Idioma | Decisión | Motivo |
|---|---|---|---|
| README_ES | ES | REFERENCIAR | Re-traducción prohibida; nota ES de 4 líneas apunta a fuente única para Trust/wizard/dry-run |
| docs-view.tsx | EN (mismo que fuente) | UNIFICAR mínimo | Superficie de usuario que debe dar confianza standalone: párrafo Trust + wizard + dry-run (wording fuente) + referencia a README/QUICKSTART. Sin componentes nuevos (ponytail) |

## 9. INVESTIGACIÓN INTERNET

N/A.

## 10. VALIDACIÓN + CIERRE

- [ ] `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps
- [ ] `git diff --check` → 0
- [ ] `Select-String` one-liners presentes en las 4 superficies
- [ ] OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json` → `ocr delegate rule`) — advisory; Critical/High bloquean
- [ ] DoD 3 niveles (Correctness: comandos idénticos a fuente; Quality: diff mínimo, sin re-traducción; Integration/Docs: referencias a fuente única)
- [ ] P2-01 orquestador (vanta-review)
- [ ] Gates D/V/C vía `question` — D: no disparado (docs-only, 2 archivos); V/C al cierre
- [ ] Commit conventional `docs:` NO PUSH, staging selectivo (solo 3 paths propios); WIP ajeno intocable
- [ ] RESULTADO §7 pipeline-full

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos, secciones foco):** `README_ES.md:235-294` · `README.md:185-234` · `docs/QUICKSTART.md:1-80` · `web/src/components/vanta/docs-view.tsx:230-309` · `scripts/validate-docs-coverage.ps1` (227L) · plan file Task 9 + recitations FIND-102/114
- **Referencias hacia dentro:** README_ES § Instalación no es referenciado por otros docs (grep: ninguna referencia a README_ES); docs-view InstallCard es autocontenido (imports: Package/Terminal/Boxes/Wrench/CodeBlock ya existentes — no se añaden imports)
- **Referencias entrantes:** fuente única README.md:192-221 + QUICKSTART §0 son el target de las nuevas referencias (lectura, no edición)
- **Veredicto:** impacto mínimo — 2 ediciones de texto aditivas (nota ES + párrafo EN), 0 cambios de lógica, 0 imports, 0 deuda. Reversible por revert del commit.

## Steps

- [x] Step 1 DISCOVERY — superficies ubicadas + decisiones por superficie + task file creado ✅ (este file)
- [x] Step 2 ACT — editar README_ES (nota referencia) + docs-view (párrafo Trust/wizard)
- [x] Step 3 VERIFY — coverage 0 gaps + diff-check + Select-String + OCR advisory + tsc
- [x] Step 4 COMMIT — staging selectivo + `docs:` commit 39000b2c, NO PUSH
- [x] Step 5 CLOSE — plan recitation + progreso + `campaign_update_task_state completed` + RESULTADO

## Spec (decisiones — Gate mecánico N/A docs, se registra igual)

| # | Decisión | Opción elegida | Evidencia |
|---|---|---|---|
| 1 | README_ES: unificar vs referenciar | REFERENCIAR | Prohibición re-traducción (§2); nota ES apunta a fuente |
| 2 | docs-view: unificar vs referenciar | UNIFICAR mínimo + referencia | Mismo idioma; superficie standalone necesita Trust visible |
| 3 | SPEC.md placeholders | NO TOCAR | Spec futura `<release-url>`, fuera del contrato |
