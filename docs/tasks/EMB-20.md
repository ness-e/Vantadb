# EMB-20 — docs (README + MCP + SKILL) — última (Wave5)

> **Plan:** `docs/plans/2026-09-16-embeddings-auto.md` (Wave5, última; finaliza tras verify EMB-19 — ya verde `a5d549af`)
> **Estado:** ⏳ IN PROGRESS 2026-09-17 (DISCOVERY completo, Steps en curso)
> **Appetite:** 4h · 🟢 · 🟢
> **Branch/Commit:** develop / `docs: EMB-20`
> **Cynefin:** 🟦 obvio · ⬆️ 0 / ⬇️ 2 steps
> **Ruta:** vanta-docs
> **SDP:** ver §5

## 1. TAREA

**Objetivo:** documentar SOLO lo verde en EMB-19 para que el próximo agente use bien los límites (FIND-67/68/83 doc-driven). Última tarea de la campaña; describe lo verificado en EMB-19 (commit `a5d549af`: rebuild final, cat 0.9158 vs 0.8427/0.8423 gap 0.0732 dim 384 fallback:false, put→get con vector, search/recall, matriz ollama/openai documentada-no-ejecutada, suites mcp 93/93 + memory verdes, test-mcp 4/4×3, audit 1 pre-existente, reinstall 79 vía PATH).

**Contrato exacto (ley):**
1. Tabla modelo→dim→idioma→tamaño→cuándo usar (9 ids del `manifest.json`, fuente Regla 11).
2. Cómo activar cada uno (3 proveedores: `local` / `ollama` / `openai` con env vars + launcher + requisitos).
3. Regla una-dim-por-base (Q4: bloquear+guiar, error con dims + comando `rebuild_index` / `reindex_hnsw_from_text`, nunca auto-reindex).
4. Nota `embed_texts` real (proveedor real + `fallback` flag + `model` param + budgeting 128/25k intacto + números EMB-19 con fuente).
5. Coverage 0 gaps (`scripts/validate-docs-coverage.ps1` verde; skills mirror hash-SAME).

**Acceptance del plan:** Gate Justificación = lo no documentado no existe para el próximo agente. Pre-mortem: documentar comportamiento no verificado → SOLO lo verde en EMB-19 (Regla 11: cada número con fuente).

## 2. ARCHIVOS

**Clave (con :línea):**
- `embeddings/README.md:1-92` (tabla 9 modelos + quickstart + env vars EMB-02 desactualizadas → curar: cuándo usar explícito + 3 proveedores + dim rule + nota real).
- `docs/api/MCP.md:282` (`embed_texts` dice "EMB-05 deterministic 384d fallback" — desactualizado → real + flag + model + budgeting) + `:212-253` (perfiles, contexto) + `:271-283` (search ops, donde vive la nota).
- `skills/vantadb-mcp/SKILL.md:116-194` (tools table + search ops sin `embed_texts` detallado → sección embeddings para el próximo agente) + mirror `.opencode/skills/vantadb-mcp/SKILL.md` (hash-SAME obligatorio, FIND-83).

**Relacionados (lectura directa, sin grep-loop):**
- `embeddings/manifest.json:1-115` (ids/dims/sizes/langs/license/group — fuente de la tabla, Regla 11).
- Decisiones Q1-Q5 del plan (`docs/plans/2026-09-16-embeddings-auto.md:26` — asistente-defaults, 3+aviso, local+ollama/openai, bloquear+guiar, fallback-avisado).
- Task files W0-W4 como fuentes — SOLO lo verde: `docs/tasks/EMB-10.md` (binario + ORT≥1.27 + señal 0.9533/0.9123), `EMB-11.md` (ORT persistente `%LOCALAPPDATA%/VantaDB/onnxruntime`), `EMB-12.md` (lanzador `vanta-mcp-local.ps1`, smoke dim==384), `EMB-13.md` (señal 0.9282 + fallback flag), `EMB-14.md` (auto-embed put→get), `EMB-15.md` (search keys `[d1,d0,d2]` + recall hybrid), `EMB-16.md` (prefijos e5 `query:/passage:`, margen 0.0812→0.1204), `EMB-17.md` (`model` switch + `MAX_CACHED_LOCAL_MODELS=2`), `EMB-18.md` (dim gate + guía honesta).
- `docs/tasks/EMB-19.md:138-162` (verify a documentar: cat 0.9158/gap 0.0732/dim384/fb:false + search/recall + matriz ollama/openai + suites + reinstall 79 + coverage 0 gaps).
- `vanta-mcp-local.ps1:1-99` (env explícita + ORT autodetect + `-DbPath` mandatory — fuente de "cómo activar local").
- `src/config.rs:203-209,586-595,936-957` (envs `VANTADB_EMBEDDING_PROVIDER` default `ollama`, `VANTADB_LOCAL_MODEL`, `VANTADB_OPENAI_API_KEY/MODEL` — fuente de "cómo activar cada proveedor").
- `src/llm.rs:17,58-93,652-723` (factory por env + OpenAI requiere key con error claro — fuente, solo lectura).
- `scripts/validate-docs-coverage.ps1:167-219` (gate 0 gaps: MCP tools 49 + skills mirror 10 pares).

**Prohibidos (WIP ajeno que NO se toca):** `.opencode/` (salvo mirror SKILL hash-SAME, que SÍ se toca parejo), `Justfile`, `completions/_vanta-cli*`, `desktop/src-tauri/Cargo.lock`, `.github/workflows/ocr-delegate.yml`, `dev-tools/ocr-review.ps1`, `reparacion.bat`, `docs/pipeline-state.json`, stash, `src/`, `vantadb-mcp/src/`, `~/.cargo/bin`, plan file salvo recitation (orquestador), `docs/Backlog.md` + `docs/avance/` (orquestador/progreso).

## 3. DEPENDENCIAS

- **Wave:** Wave5 (Wave0-W4 DONE + EMB-19 ✅ `a5d549af`). Última tarea; luego cierre: progreso masivo + retrospectiva + archive (orquestador).
- **BLOQUEANTES:** EMB-19 (docs finalizan tras su verify — ya verde 2026-09-17).
- **Previa:** EMB-19 ✅ / **nextTask:** ninguna (última).
- **Hereda:** manifest, Q1-Q5, task files W0-W4 (fuentes solo-verde), EMB-19 verify.

## 4. REFERENCIAS

- `documentation-and-adrs` (CARGADA: documentar porqué/límites, no restatar código; gotchas donde importan).
- `writing-guidelines` (CARGADA: voz/tono, tablas claras; guidelines externas sin red → se aplica criterio local + se anota).
- Regla 11 (cada número con fuente; documentar SOLO lo verde en EMB-19).
- `docs/api/MCP.md:18,218` (LEÍDA: stdio JSON-RPC 2.0, perfil `full` default 79 tools).
- Matriz REAL/PARCIAL de la Propuesta (retrieval híbrido REAL; `extract_skills` PROPUESTA fuera de scope — no documentar como existente).
- Tabla Spec N/A (docs, sin símbolo público nuevo; si aparece símbolo nuevo en verify → por-evidencia + fila FIND, no silencio).

## 5. SKILLS

**SDP (campaign_discover_skills_v2 phase=BUILD archivosClave="embeddings/README.md,docs/api/MCP.md,skills/vantadb-mcp/SKILL.md" contractKeywords=["embeddings-readme","mcp-docs","skill-docs","coverage-gaps"] maxSkills=8 — ejecutado 2026-09-17):**
- `campaign-executor` (1.00 base MCP) — cuándo: state machine PLAN→ACT→VERIFY + RESULTADO §7.
- `source-driven-development` (1.00 base MCP) — cuándo: cada env/comando/tabla verificados en código (`config.rs`, `server.rs`, `manifest.json`), no de memoria.
- `incremental-implementation` (1.00 lifecycle BUILD) — cuándo: slices verticales delgados (README → MCP.md → SKILL+mirror → verify → commit).
- `test-driven-development` (1.00 lifecycle BUILD) — cuándo: N/A en docs (sin lógica nueva); el "test" es `validate-docs-coverage.ps1` 0 gaps + `git diff --check`.
- `context-engineering` (1.00 lifecycle BUILD) — cuándo: sesión multi-evidencia (este file = context pack).
- `doubt-driven-development` (1.00 lifecycle BUILD) — cuándo: stakes (documentar lo no-verificado es peor que no documentar) → solo-verde EMB-19.
- `api-and-interface-design` (1.00 lifecycle BUILD) — cuándo: superficie MCP (`embed_texts` shapes, `fallback`/`model` campos) descrita sin inventar.
- `documentation-and-adrs` (keyword-mapped) — cuándo: tablas/límites/gotchas + decisiones con tradeoff.
- `writing-guidelines` (keyword-mapped) — cuándo: revisar tablas/tono contra guías.
- `spec-driven-development` (keyword-mapped) — cuándo: doc-driven (FIND-67/68/83): el doc es el contrato del próximo agente.
- `progreso` (base sesión) — cuándo: al cierre (orquestador; esta tarea NO toca Backlog/avance directamente).
- `frontend-ui-engineering` (sugerida por scorer) — DESCARTADA: sin `web/`, scope discipline.
- **Cargadas vía `skill`:** documentation-and-adrs, writing-guidelines, source-driven-development (+ spec-driven-development referenciada sin recarga redundante; campaign-executor/progreso = base sesión).
- **SDP registrado:** `SDP: campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design, documentation-and-adrs, writing-guidelines, spec-driven-development, progreso (frontend-ui-engineering descartada: sin web/)`

## 6. HERRAMIENTAS+MCP

- `pwsh -NoProfile scripts/validate-docs-coverage.ps1` (contrato: 0 gaps; MCP tools 49 + skills mirror 10 pares).
- `git diff --check` (whitespace) + `cargo fmt --check` (0 prod tocado → debe pasar en verde heredado).
- `campaign_verify_cmd` (bug exit -1 conocido → bash directa + mención en RESULTADO) + `campaign_validate_command` antes del primer comando riesgoso + `campaign_enforce_state` en transiciones + `campaign_session_track` multi-iteración.
- MCP a usar: `check_index_coverage` si aplica a docs (docs no indexadas → lectura directa) + `session_track` — sin grep-loop si lectura directa responde (responde).
- Commit SOLO propios: `embeddings/README.md` + `docs/api/MCP.md` + `skills/vantadb-mcp/SKILL.md` + `.opencode/skills/vantadb-mcp/SKILL.md` (mirror) + `docs/tasks/EMB-20.md`. WIP ajeno EXCLUIDO.

## 7. INVESTIGACIÓN CÓDIGO

**Blast radius resumido (construido en DISCOVERY 2026-09-17):**
- `embeddings/README.md` (tabla 9 + quickstart + env EMB-02 desactualizadas) → `docs/api/MCP.md` (`embed_texts` EMB-05 desactualizado + sin sección providers/dim-gate) → `skills/vantadb-mcp/SKILL.md` (sin sección embeddings; el próximo agente lee ESTA primero) + mirror hash-SAME.
- Implicaciones: SKILL es la que lee el próximo agente (FIND-67/68/83); si SKILL omite `fallback`/`model`/dim-gate, el agente romperá su base en silencio. README sirve al operador humano (qué modelo descargar, cuánto pesa, cuándo usar). MCP.md sirve al integrador (shapes + envs + errores accionables).
- Riesgo: documentar comportamiento no verificado (p.ej. openai-real sin key, qwen3 ONNX que no existe, bench sin baseline) → mitigación: SOLO verde EMB-19 + marcar `documentado-no-ejecutado` donde aplique (ollama-sin-modelos/openai-sin-key, precedente FIND-69) + cada número con fuente (Regla 11).

**Impacto mapeado (Regla 0):**
- **Archivos leídos (completos):** plan §EMB-20 + EMB-19 (192L) + EMB-13/16/17/18 + `embeddings/README.md` (92L) + `manifest.json` (115L) + `docs/api/MCP.md` (`:18,198,212-283,445-447`) + `skills/vantadb-mcp/SKILL.md` (599L) + `vanta-mcp-local.ps1` (99L) + `validate-docs-coverage.ps1` (227L) + grep `VANTADB_*` en `src/config.rs`/`llm.rs`.
- **Hacia dentro:** manifest (fuente tabla), `config.rs`/`llm.rs` (fuente providers), EMB-19 §Ejecución (fuente números), `vanta-mcp-local.ps1` (fuente launcher), Q1-Q5 (fuente decisiones).
- **Entrantes:** EMB-20 es hoja (nadie depende de ella salvo el próximo agente FIND-67/68/83 + cierre de campaña).
- **Veredicto:** BAJO (solo docs, 0 prod). Rollback = revert 1 commit `docs:`.

## 8. INVESTIGACIÓN PROBLEMA

- Hipótesis de gaps (qué falta para que el próximo agente use bien los límites): (a) `fallback` flag — sin él el agente cree que tiene señal cuando tiene hash; (b) dim gate — sin él mezcla 384/768 en silencio y el ranking se corrompe (AUD-046); (c) `model` param — sin él pide `bge-base` y recibe default sin aviso; (d) providers — sin matriz el agente no sabe que `ollama` default sin servidor = fallback ni que `openai` exige key de sesión nunca a disco; (e) e5 prefijos — sin nota el agente mide margen bajo y culpa al modelo; (f) una-dim-por-base — sin comando de regeneración el agente corre `rebuild_index` solo y espera cambio de dim (mentira útil EMB-18).
- Decisión spec con tradeoff: tabla completa 9 modelos (incl. qwen3 excepción GPU-only sin ONNX) vs solo 3 en disco — se documentan los 9 con estado `en disco / bajo demanda / excepción` porque Q2 lo promete (3 verificados + resto avisado con tamaño/tiempo); el aviso de tamaño/tiempo ES el contrato Q2.

## 9. INVESTIGACIÓN INTERNET

N/A — todo local (manifest + envs + suites + MCP stdio + scripts en repo; EMB-19 ya verificó sin red). Sin ambigüedad externa real (model card e5 ya verificada en EMB-16 §8 con HTTP 200; no se re-cita como evidencia nueva). Sin red usada como evidencia → sin citas, sin deuda TSYS-13. Gate de citas TSYS-13: URLs en esta tarea son internas al repo (`manifest.json`, task files) → N/A. Si surgiera duda de API MCP spec → `webfetch` docs oficiales + deuda TSYS-13.

## 10. VALIDACIÓN+CIERRE

- [ ] Step 1 — `embeddings/README.md`: tabla con cuándo-usar + 3 proveedores + dim rule + nota real con números EMB-19 (cada número con fuente).
- [ ] Step 2 — `docs/api/MCP.md`: `embed_texts` real + sección embeddings (providers/model/auto-embed/query/dim-gate) sin romper paridad 49 tools.
- [ ] Step 3 — `skills/vantadb-mcp/SKILL.md` + mirror hash-SAME: sección embeddings para el próximo agente (límites operativos).
- [ ] Step 4 — verify full relevante: `validate-docs-coverage.ps1` 0 gaps + `git diff --check` + `cargo fmt --check` (heredado) → OCR advisory (`dev-tools/ocr-review.ps1` si existe; Critical/High=bloquea, Medium→FIND-*) → DoD 3 niveles → reviewer distinto P2-01 → commit `docs: EMB-20` SOLO propios → recitation + RESULTADO §7 + Save Point.
- Gates D/V/C activos: D (solo docs, sin símbolos nuevos → NO dispara); V (2 fallas mismo-error → `question`); C (colaterales → fila FIND + `question`).
- DoD 3 niveles: **Task** (contrato 5 puntos verificable) · **Commit** (atómico `docs: EMB-20`, `git diff` solo propios, coverage 0 gaps) · **Release** (N/A docs; pre-push gate Regla 1 en EMB-19 ya verde).

## Steps

- [x] **Step 1 — `embeddings/README.md`:** cuándo-usar por modelo + cómo activar (3 proveedores con envs + launcher + ORT≥1.27) + regla una-dim-por-base + nota `embed_texts` real con números EMB-19 · Verify: `git diff --check` + lectura ✅ DONE 2026-09-17
- [x] **Step 2 — `docs/api/MCP.md`:** `embed_texts` real (`fallback`/`model`/budgeting) + §Embeddings (providers + auto-embed + query mismo-proveedor + dim-gate + e5 nota) · Verify: `validate-docs-coverage.ps1` (49 tools intacto) ✅ DONE 2026-09-17
- [x] **Step 3 — SKILL + mirror + cierre:** sección operativa en `skills/vantadb-mcp/SKILL.md`, mirror idéntico a `.opencode/skills/vantadb-mcp/SKILL.md`, verify full + OCR + commit `docs:` + RESULTADO · Verify: `validate-docs-coverage.ps1` 0 gaps (10 pares SAME) + `git diff --check` ✅ DONE 2026-09-17

## Ejecución (evidencia mecánica 2026-09-17)

- `scripts/validate-docs-coverage.ps1`: 8/8 verde — sdk 28, config 60, cli 42, py 48, mcp-tools **49**, skills-mirror **10 pares hash-SAME** → **0 gaps** ✅ (baseline pre-cambio también 0 gaps; sin regresión).
- Mirror: `Get-FileHash` SHA256 `skills/vantadb-mcp/SKILL.md` == `.opencode/skills/vantadb-mcp/SKILL.md` (`FCF291C5…DAF3E`) ✅ FIND-83.
- `git diff --check` sobre los 5 paths propios: limpio (solo warnings CRLF pre-existentes ajenos) ✅.
- `cargo fmt --check`: exit 0 ✅ (0 prod tocado).
- OCR advisory (`dev-tools/ocr-review.ps1 -Format text`): mis 4 paths `unsupported_ext` (md, igual que EMB-12/19) → **sin Critical/High, sin Medium → sin FIND-*** ✅. Reviewables del preview son WIP ajeno (completions/Justfile/ocr-delegate.yml) — excluidos del commit.
- Números documentados con fuente (Regla 11): cat `0.9158 vs 0.8427/0.8423 gap 0.0732 dim 384 fallback:false model=multilingual-e5-small` (EMB-19 Step 1); precedentes EMB-13 `0.9282 vs 0.8427/0.8366`, EMB-10 `pares≥0.91 vs impares≤0.78`; margen e5 `0.1204 vs 0.0812 +48%` (EMB-16); search `["d1","d0","d2"]` + recall `hybrid` (EMB-15/19); suites `93/93 mcp + memory verdes`, `test-mcp 4/4×3`, `tools/list 79 vía PATH`, bins `24.83/26.26MB`, audit 1 pre-existente `RUSTSEC-2026-0253` (todos EMB-19 Steps 1-3, commit `a5d549af`).
- Matriz parcial documentada como tal (FIND-69): ollama degradación avisada (servidor v0.33.2 vivo, `models:[]`, `404`) + openai degradación avisada (sin key) + embedding real documentado-no-ejecutado. Qwen3 excepción GPU-only `onnx:null` marcada como tal (manifest + README previo).

## Spec Gate

Sin símbolos públicos nuevos (docs, 0 `pub fn`/tool/endpoint). Tabla N/A por ser docs. Gate D: blast 3 docs + 1 mirror, sin hot path, sin API pública nueva, contrato mecánico → NO dispara `question` (motivo: solo-docs + solo-verde-EMB-19). Gate V: 2 fallas mismo-error → `question` antes de FAILED. Gate C: colaterales → fila FIND + `question` (WIP ajeno en worktree → excluir del commit, confirmar alcance).

## Context Save Point

DISCOVERY completo 2026-09-17. Plan + Q1-Q5 + EMB-10..19 (solo-verde) + README (92L) + manifest (9 ids) + MCP.md (`:18,218,282`) + SKILL (599L, sin sección embeddings) + launcher (99L) + envs (`config.rs`/`llm.rs` vía grep) + coverage baseline 0 gaps + WIP ajeno (11 files, 0 solape). Task file creado (este archivo). Siguiente: Step 1 README.

## Invariantes de dominio (handoff — MUST)

- **Preservar:** SOLO verde EMB-19 (Regla 11, cada número con fuente); shapes MCP intactos en la doc (`embeddings/model/dim/dimensions/fallback/warning/count/truncated/next_cursor`); budgeting 128/25k; fallback Q5; mensajes EMB-18 exactos (no parafrasear el Hint); prefijos EMB-16 por familia; `MAX_CACHED_LOCAL_MODELS=2`; secrets NUNCA a disco (keys solo env de sesión); WIP ajeno excluido del commit; sin push (vía vanta-lead/orquestador); DB temporal para ejemplos (NUNCA base del usuario).
- **Comandos:** `pwsh -NoProfile scripts/validate-docs-coverage.ps1` + `git diff --check` + `cargo fmt --check`.
- **Deuda:** openai-real no ejecutado (sin key, FIND-69) → documentado-no-ejecutado, NO deuda oculta; vanta-review si no disponible = deuda leve; `references/api-reference.md` si menciona `embed_texts` dummy → NOTICED BUT NOT TOUCHING (fuera de scope, candidato FIND del orquestador).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto:** 0 (solo docs, 0 código). Si el verify revela bug de código → Prove-It + fix mínimo NO aquí (nace FIND-*, este task no toca `src/` ni `vantadb-mcp/src/`).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato §1 (5 puntos) verificable + `validate-docs-coverage.ps1` 0 gaps + `git diff --check` limpio |
| **Commit** | Atómico `docs: EMB-20` (solo 5 paths propios incl. mirror), `git diff` solo propios, fmt heredado verde |
| **Release** | N/A (docs; pre-push gate Regla 1 ya verde en EMB-19) |

SKILLS_CARGADAS: campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design, documentation-and-adrs, writing-guidelines, spec-driven-development, progreso
SDP: campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design, documentation-and-adrs, writing-guidelines, spec-driven-development, progreso (frontend-ui-engineering descartada: sin web/)
