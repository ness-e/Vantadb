# EXE-03-prep: kit gate Fase A listo (la ejecución con humanos es owner-side)

## Metadata

- **Plan file:** `docs/dev/plans/2026-09-19-publicacion.md` (Task 6, Wave2 última)
- **Fuente:** `docs/dev/Backlog.md` fila `EXE-03` + `docs/dev/research/archive/Investigacion-plan.md` Fase A (A.1/A.2/A.3) + plan Task 6
- **Esfuerzo:** 🟢 1d (plan: 1d / 🟢)
- **Prioridad:** 🟠 Media-Alta
- **Tipo:** Docs
- **Turns estimados:** 5-10
- **Creado:** 2026-09-19
- **last-synced:** 2026-09-19
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps restantes
- **Commit:** `0a7f84b2` (docs: EXE-03-prep, 2 files, NO PUSH)
- **Branch:** develop · **Commit:** `docs: EXE-03-prep — ...` (NO PUSH, staging selectivo)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/DISTRIBUTION.md` §4 (cita EXE-03 como gate del anuncio) · `docs/dev/Backlog.md` fila EXE-03 (ejecución humana, no este kit) · plan file Task 6 |
| Callees | `docs/dev/research/archive/Investigacion-plan.md` Fase A (spec fuente: A.1/A.2/A.3) · `README.md` (auditoría lectura) · `docs/user/QUICKSTART.md` (flujo testers) · `docs/api/MCP.md` (superficie 87 tools) · `docs/user/operations/EXPERIMENTAL_FEATURES.md` (boundary) |
| Implicaciones | contrato docs-only: 0 cambios de comportamiento público, 0 API, 0 performance, 0 migración, 0 tests afectados; `validate-docs-coverage.ps1` no cubre el archivo nuevo (chequea SDK/config/error/CLI/Python/MCP contra sus docs fijos) → 0 gaps esperados |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/dev/plans/2026-09-19-publicacion.md` (Task 6) · `README.md` (362L) · `docs/user/QUICKSTART.md` (254L) · `docs/DISTRIBUTION.md` (134L) · `docs/dev/research/archive/Investigacion-plan.md` Fase A (L11-36 + checklist README L146-156) · `scripts/validate-docs-coverage.ps1` (227L) · `.opencode/references/definition-of-done.md` · `.opencode/rules/README.md`
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** kit nuevo referencia (no edita): README, QUICKSTART, DISTRIBUTION, MCP.md, EXPERIMENTAL_FEATURES.md, CONTRIBUTING.md, FAQ, CHANGELOG, Backlog EXE-03
- **Archivos que referencian a los editados (referencias entrantes):** `docs/FASE-A.md` no existe (cero entrantes) · `docs/dev/tasks/EXE-03-prep.md` no existe (cero entrantes) · grep `FASE-A` en repo: 0 hits (solo DISTRIBUTION menciona "Fase A" genérico, no el path)
- **Veredicto impacto:** bajo — 2 archivos nuevos bajo `docs/`, 0 ediciones a archivos existentes (README solo lectura: honesto, sin fix). Si el kit desaparece no se rompe nada (docs huérfana, sin imports de código).

## Contrato

"Kit completo en `docs/FASE-A.md`: checklist Fase A imprimible (producto/calidad/docs/comunidad/comunicación, todo-SÍ) + plantilla usuario-01 por persona + README auditado honesto (veredicto + evidencia) + instrucciones testers; `pwsh scripts/validate-docs-coverage.ps1` exit 0 (0 gaps) + `git diff --check` limpio"

## Spec (SDD — Phase 1b)

N/A justificado por evidencia (tarea 100% docs, cero decisiones técnicas abiertas):
- Señales feature-add verificadas una por una contra el plan de solución: sin `pub fn`/struct nuevo (no se toca `src/`) · sin tool MCP/endpoint nuevo (solo se cita la superficie 87 existente) · sin método de binding nuevo · sin componente web nuevo · sin capability nueva (el kit prepara, no ejecuta). Ninguna aplica → no es feature-add → sin Gate P/D, sin tabla de decisiones.
- Única fuente de spec: `Investigacion-plan.md` Fase A (A.1/A.2/A.3) + fila Backlog EXE-03 — ya decididos, se transcriben (no se re-deciden). Una sola fuente (`docs/FASE-A.md`), cero duplicación de plantillas (pre-mortem 2).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** no ejecutar el gate con humanos (owner-side) · no publicar/anunciar nada (posts pausados hasta Fase A en verde, decisión 2026-09-08) · WIP ajeno intocable (`completions/*`, `.opencode`, `reparacion.bat`, stash GOV-C4, `C:/Users/Eros/.vantadb*`, `desktop/src-tauri/Cargo.lock`) · plan file solo recitation al cierre · `docs/dev/Backlog.md` intacto · staging selectivo (solo los 2 archivos de esta task)
- **Comandos de verificación:** `pwsh scripts/validate-docs-coverage.ps1` (exit 0, 0 gaps) · `git diff --check` (limpio) · `git status --short` (solo 2 untracked/modified de esta task en staging)
- **Deuda pendiente:** ninguna (kit docs; la ejecución humana EXE-03 queda owner-side fuera de esta task)

## Recitation (canónico — estructura única)

contract:
  verificacion: `pwsh scripts/validate-docs-coverage.ps1` exit 0 + `git diff --check` limpio (al cierre)
  evidencia:
    - claim: gate Fase A definido como A.1 stranger-test + A.2 checklist todo-SÍ + A.3 objetivo 5 personas
      evidencia: docs/dev/research/archive/Investigacion-plan.md:11-36 + docs/dev/Backlog.md fila EXE-03
      confianza: alta
    - claim: superficie MCP real 87 tools (49 core + 37 extendidas + contexto)
      evidencia: vantadb-mcp/src/handlers/tools.rs:26-30 (50+37) + :1018-1019 (extends scenes/threads) + docs/api/MCP.md:198,220
      confianza: alta
    - claim: versiones idénticas 0.5.0 código/paquete/docs
      evidencia: Cargo.toml:728 + vantadb-python/pyproject.toml:7 + vantadb-ts/package.json:3
      confianza: alta
    - claim: README sin claims falsos (cero features v1.0.0, cero conteos, boundary honesto)
      evidencia: README.md:169-184 (Product Boundary) vs docs/user/operations/EXPERIMENTAL_FEATURES.md
      confianza: alta
  artefactos:
    - docs/FASE-A.md
    - docs/dev/tasks/EXE-03-prep.md
  invariantes: kit-only, owner-side la ejecución; prohibidos intactos (ver Invariantes)
  deuda: ninguna
  queda_pendiente: P2-01 batch + progreso + archive los hace el orquestador al cierre del plan (no esta task)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — 2 archivos docs nuevos, 0 código, 0 deuda nueva, 0 pago requerido.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Aplica |
|-------|------|--------|
| **Task** | contrato verificable (kit + coverage 0 gaps + diff limpio) | ✅ sí |
| **Commit** | atómico (~100L docs), `docs:` conventional, staging selectivo, verify mecánica | ✅ sí |
| **Release** | N/A-justificado: docs-only sin cambio user-visible de código (CHANGELOG lo toca release-plz; no se edita) | N/A |

DoD standing: Correctness (AC a-d del contrato) · Quality (una sola fuente, sin duplicar plantillas) · Integration (referencias a docs existentes verificadas) · Documentation (el entregable ES documentación) · Ship-readiness (rollback = `git revert` del commit docs; sin riesgo).

## Herramientas necesarias

- grep (auditoría README-vs-superficie real) + `pwsh scripts/validate-docs-coverage.ps1` + `git diff --check` + `campaign_verify_cmd` (BUG exit -1 conocido → bash directa + mención en RESULTADO)
- codegraph N/A (docs, sin símbolos) · Cargo N/A · Internet N/A

**Skills cargadas (SDP):** `documentation-and-adrs` — kit/checklist/plantilla es documentación de gate (cuándo: checklist imprimible + instrucciones testers + honestidad de límites); resto del SDP (`campaign-executor` base, `incremental-implementation`, `test-driven-development`, `context-engineering`, `source-driven-development`, `doubt-driven-development`, `frontend-ui-engineering`, `api-and-interface-design`) evaluado: lifecycle genérico sin justificación por contrato (ninguno aplica a docs-only sin código/UI/API nueva) → no cargados. `SDP: documentation-and-adrs + base (keywords: fase-A, stranger-test, checklist, kit, honestidad, docs)`.

## Investigation Notes

- **DISCOVERY = auditoría docs (código N/A):** README (362L) leído completo — veredicto HONESTO sin fix: cero claims v1.0.0/entity-resolution/conflict-detection (grep `v1\.0|entity resolution|production-ready` en README = 0 hits); cero conteos de tools; Product Boundary (L169-184) idéntico a EXPERIMENTAL_FEATURES.md (producción/optativo/experimental/deferred); versiones 0.5.0×3 idénticas; Discord canónico `g8nqB3NtXt` (L41) + FAQ (L66) + CHANGELOG (L347) + LICENSE (L360-362) presentes.
- **Gap honesto registrado (no fix README):** README no lleva banner explícito "Early Access" ni sección "cómo reportar" (A.2 los exige). Omisión, no falsedad → `fix solo si deshonesto` = NO editar README; el kit lo declara como ítem ⬜ del checklist con acción owner (añadir banner) en vez de inventar estado.
- **Benchmarks README (L282-309):** números con hardware citado (12-core/AVX2, Ryzen `-C target-cpu=native`) + comando reproductor (`benchmarks/vanta_benchmark_report.json` vía `vantadb_local_bench.py`) → Regla 11 OK, no se tocan.
- **Plantilla usuario-01 fuente única:** `Investigacion-plan.md:20-24` (Fecha/OS/Python-Node/Tiempos/Bloqueos/Preguntas/Errores/Cambios) — se transcribe ampliada, no duplicada.
- **No duplicar:** DISTRIBUTION §4 ya tiene announcement-checklist (PREPARE-only) → FASE-A.md lo referencia, no lo copia (pre-mortem 2).
- **Frontmatter obligatorio:** FIND-126 (Check Frontmatter exige `title:` en `docs/**/*.md`) → `docs/FASE-A.md` lleva frontmatter completo.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — spec Fase A cerrado (A.1/A.2/A.3), superficie 87 verificada, versiones 0.5.0×3 |
| Pendientes de ejecución (downhill) | 2 — Step 1 kit + Step 2 verify/commit/cierre |
| % completado | 40% (discovery + task file) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — N/A-justificado: docs-only, cero trust boundaries (sin input de usuario, sin auth, sin dependencias, sin storage/FFI/red, sin secrets). Nada que auditar con `security-and-hardening`.
- [x] **PERFORMANCE** — N/A-justificado: cero hot paths (sin `vector/`, `engine.rs`, loops, serialización). Sin baseline que medir.

## Steps

### Step 1: Kit `docs/FASE-A.md` completo

- **Archivos:** `docs/FASE-A.md` (nuevo, ~150L) + este task file (sync estados)
- **Acción:** crear kit fuente-única: checklist Fase A imprimible (5 familias A.2, todo-SÍ) + plantilla usuario-01 por persona (A.1 ampliada) + instrucciones testers (instalar + QUICKSTART + reportar) + veredicto README honesto con evidencia + handoff owner (A.3, 5 personas) + referencias (no duplicar DISTRIBUTION §4)
- **Verify:** existe + frontmatter `title:` + grep `usuario-01` ≥1 + grep `todo SÍ o no se anuncia` ≥1
- **Estado:** ✅ COMPLETED (`docs/FASE-A.md` 77L, frontmatter + ficha usuario-01 + checklist 5 familias)

### Step 2: Verify + commit + cierre

- **Archivos:** `docs/FASE-A.md`, `docs/dev/tasks/EXE-03-prep.md` (solo estos 2 en staging)
- **Acción:** `pwsh scripts/validate-docs-coverage.ps1` (0 gaps) + `git diff --check` + `git status --short` (staging selectivo, WIP ajeno fuera) → commit `docs: EXE-03-prep — ...` (NO PUSH) → `campaign_update_task_state completed` + recitation plan file (solo recitation, sin editar cuerpo) → RESULTADO §7
- **Verify:** commit hash existe + coverage exit 0 + diff --check limpio
- **Estado:** ✅ COMPLETED (commit `0a7f84b2` 2 files/270+ · coverage exit 0 0 gaps · diff --check limpio · WIP ajeno restaurado a staged)

## Dependencias

- Task previa: PROV-12 ✅ (Wave2 en secuencia, archivos disjuntos packaging-vs-kit) — cerrada por orquestador
- Bloqueantes: ninguno
- NextTask tras cierre: ninguna — ÚLTIMA del plan Publicación (cierre del orquestador: P2-01 batch + progreso + archive)

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta el orquestador en batch al cierre del plan (no esta task).

- **Revisor:** orquestador (P2-01 batch del plan) — esta task no auto-revisa
- **Enfoque:** ¿kit completo y fuente-única? ¿README veredicto evidenciado?
- **Cómo se probó:** coverage 0 gaps + diff limpio + commit atómico (evidencia mecánica, no auto-reporte)
- **Checklist anti-hábitos tóxicos:**
  - [ ] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [ ] No saltarse la clarificación por "ya sé qué quiere".
  - [ ] No declarar done sin verificar contra los acceptance criteria.
  - [ ] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [ ] No hacer un solo intento de búsqueda y darlo por saturado.
  - [ ] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [ ] No reintentar en bucle sin diagnóstico.
  - [ ] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [ ] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [ ] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** pendiente (orquestador)

## Notas

- Gate D: no dispara (docs-only, 1 archivo nuevo, contrato cerrado, cero símbolos públicos nuevos) — sin `question`.
- Gate V: no dispara (sin fallos verify) — sin `question`.
- Gate C: no dispara (cero colaterales: staging selectivo deja WIP ajeno intacto) — sin `question`.
- OCR delegation: N/A-justificado docs-only (sin código que revisar con `ocr-review.ps1`).
- Queda fuera (owner-side): conseguir 5 personas, correr stranger-tests, veredicto GO/NO-GO del anuncio.
