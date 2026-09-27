# DEF-04: Naming freeze 0.7.0→1.0 (ADR + política de alias)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 12)
- **Fuente:** Backlog `DEF-04` (L939) · master Task 12 · R1 (congelar pre-usuarios es gratis)
- **Esfuerzo:** 🟢 1d
- **Prioridad:** 🔴
- **Tipo:** Docs (ADR + política)
- **Turns estimados:** 8
- **Creado:** 2026-09-26T19:55
- **last-synced:** 2026-09-26T19:55
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 1 abierta — firma del owner (Regla 5; status `accepted` solo con firma)
- **Pendientes (downhill):** 4 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/api/VERSIONING.md` (referencia desde §Deprecation policy) · `docs/api/DEPRECATIONS.md` (HARD-01 — política de alias) · READMEs que citan ADR-030 (`README.md:80`, `README_ES.md:79`, `docs/api/PYTHON_SDK.md:79`, `vantadb-ts/README.md:163`) · LEG-01 (owner — dictamen de marca) · MKT-18f (publicación adapters con nombres congelados) |
| Callees | `ADR-030` (evidencia de marca) · `ADR-041`/`ADR-042` (precedente de renames/anti-stutter) · `docs/dev/_templates/adr.md` · `VERSIONING.md` §Deprecation policy |
| Implicaciones | Doc-only; **cero renames** (ADR-030 regla práctica) · no cambia nada publicado (crates.io `vantadb`, PyPI `vantadb-py`, npm `vantadb`) · ADR queda `proposed` hasta firma owner |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/dev/architecture/adr/ADR-030-brand-identity-naming-convention.md` (104L) · `docs/api/VERSIONING.md` (102L) · `docs/dev/_templates/adr.md` · `docs/dev/Backlog.md` (fila `DEF-04`, L939)
- **Archivos referenciados hacia dentro (imports/dependencias):** ADR-030 referencia `docs/dev/_templates/adr.md` (template) y estilo ADR-027/030 (`ADR-031:205`); DEF-04 depende del template para el draft
- **Archivos que referencian a los editados (referencias entrantes):** ADR-030 citada por `README.md:80`, `README_ES.md:79`, `docs/api/PYTHON_SDK.md:79`, `vantadb-ts/README.md:163`, `CHANGELOG.md:1925`, `ADR-031:205` — **no se editan** (solo el ADR nuevo las referencia)
- **Veredicto impacto:** **bajo** — 1 archivo nuevo (ADR) + 1 referencia en `VERSIONING.md`; sin código; `DEPRECATIONS.md` no existe aún (lo crea HARD-01 — solo coordinar).

## Contrato
"ADR con los 9 artefactos congelados (vantadb, vantadb-py, vantadb-node, vantadb-ts, vantadb-server, vantadb-mcp, vanta-cli, vanta-proxy, vanta-memory) + política de alias/deprecación (alias con fecha de remoción, nunca rename silencioso) Y firma owner (Regla 5) Y referencia desde VERSIONING.md"

## Spec (SDD — decisión con tradeoffs)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Numeración/formato del ADR | A) `ADR-045-naming-freeze.md` (siguiente libre tras ADR-044; convención `ADR-NNN-slug`) + template `_templates/adr.md` / B) esquema legacy `NNN_titulo.md` | A | ✅ decidido-por-evidencia (verificar siguiente libre al ejecutar; convención existente) |
| 2 | Relación con ADR-030 (`proposed`) | A) ADR-045 referencia/extiende ADR-030 (ADR-030 = convención de marca; ADR-045 = freeze 0.7→1.0 + alias) / B) ADR-045 supersede ADR-030 | A (owner confirma) | ⏳ Gate P firma (Step 3) |
| 3 | Política de alias | A) alias siempre con fecha de remoción + ventana ≥1 MINOR (VERSIONING §Deprecation policy) + nunca rename silencioso / B) sin política formal | A | ✅ decidido-por-evidencia (master Task 12 contrato + `VERSIONING.md:89-94`) |
| 4 | Rol IA vs owner (Regla 5) | A) IA: evidencia + draft; owner: articula Context/Decision/Consequences + firma / B) IA redacta completo y auto-accept | A | ✅ decidido-por-evidencia (AGENTS Regla 5 + master pre-mortem F1) |
| 5 | Decisiones ADR-030 pendientes (dominio, PyPI ownership, `vantadb-node` publish, case repo) | A) referenciar (no resolver acá; LEG-01 owner) / B) resolver en este ADR | A | ✅ decidido-por-evidencia (ADR-030 §Decisiones pendientes; LEG-01 en carril owner) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) **cero renames** de artefactos publicados (ADR-030 regla práctica); (2) ADR `proposed` hasta firma del owner — la IA nunca lo marca `accepted`; (3) todo alias con fecha de remoción; (4) no editar READMEs/CHANGELOG (solo referenciar); `DEPRECATIONS.md` es de HARD-01.
- **Comandos de verificación:** `rg -n "ADR-045" docs/api/VERSIONING.md` (referencia presente) · `git diff --name-only` = solo ADR nuevo + `VERSIONING.md` (+ task file) · `pwsh scripts/validate-docs-coverage.ps1` → exit 0.
- **Deuda pendiente:** cross-link a `DEPRECATIONS.md` (coordinación HARD-01 — el archivo no existe hasta que HARD-01 lo cree).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|---|---|
| `activeGoal` | `# DEF-04: Naming freeze 0.7.0→1.0 (ADR + política de alias)` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Step 1 ⬜ PENDING (evidence pack de los 9 artefactos) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia (comandos ejecutados) |
| `nextTask` | DEF-05 |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (task doc-only; no introduce deuda nueva).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable ✅ (ADR con 9 artefactos + política de alias + firma owner + referencia desde VERSIONING.md) |
| **Commit** | 1 commit atómico `docs:` (ADR + VERSIONING + task file), `git diff` limpio, verificación mecánica registrada (nunca auto-reporte) |
| **Release** | N/A justificado (policy doc pre-release; el freeze se materializa vía release-plz en 0.8.0/1.0) — ver Notas |

## Herramientas necesarias
- `question` (Gate P / firma owner — Step 3; sin firma → queda `proposed`, STOP)
- `rg`/`Get-ChildItem` (numeración ADR + estado de registries) · `pwsh scripts/validate-docs-coverage.ps1`
- `pwsh dev-tools/ocr-review.ps1` (input del review P2-01)

**Skills cargadas (SDP):** `SDP: campaign_discover_skills_v2 phase=BUILD → campaign-executor, documentation-and-adrs, deprecation-and-migration, api-and-interface-design, spec-driven-development, writing-guidelines`
- `campaign-executor` — flujo pipeline/task system (base).
- `documentation-and-adrs` — estándar ADR + template.
- `deprecation-and-migration` — política de alias/ventana de remoción (keyword mapping).
- `api-and-interface-design` — congelar nombres = contrato de superficies.
- `spec-driven-development` — tabla Spec.
- `writing-guidelines` — prosa técnica.

## Investigation Notes
- **9 artefactos a congelar:** `vantadb` (crate) · `vantadb-py` (PyPI) · `vantadb-node` (npm nativo) · `vantadb-ts` (repo dir) · `vantadb-server` · `vantadb-mcp` · `vanta-cli` · `vanta-proxy` · `vanta-memory` (master Task 12 + Backlog L939).
- **ADR-030 (status `proposed`):** tabla de 11 superficies/artefactos con evidencia live 2026-08-25 (crates.io/PyPI/npm/GitHub/DNS) + 5 decisiones pendientes del owner (dominio, PyPI ownership `DevpNess`→`ness-e`, publicar `vantadb-node`, case del repo, metadata PyPI). Regla práctica: "no renames; cada ecosistema conserva el nombre publicado".
- **Numeración:** máximo actual = `ADR-044` (`ADR-044-acumulado-breaking-0.6.0-find-133.md`); propuesto: `ADR-045-naming-freeze.md` (verificar libre al ejecutar).
- **Precedente:** ADR-041 (error variant renames) y ADR-042 (pub signatures major) — cómo el repo trató renames pre-0.7; API-01..09 completadas = último breaking pre-lanzamiento (base lista).
- **VERSIONING.md:** §Deprecation policy (L89-94): deprecación ≥1 MINOR + removal con `feat!:`/`BREAKING CHANGE:`; §Enforcement: `validate-docs-coverage` + tests; `DEPRECATIONS.md` referenciado solo en el master (lo crea HARD-01).
- **Coordinaciones:** HARD-01 (mismo F0; `DEPRECATIONS.md` + `COMPATIBILITY.md`) · LEG-01 (owner — dictamen de marca informa display/repo, no bloquea el freeze de nombres de artefactos) · HIG-02 (firma waiver ADR-041 — carril owner, contexto).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 1 — firma del owner (Regla 5) |
| Pendientes de ejecución (downhill) | 4 — Steps 1–4 |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — no aplica: doc-only, sin trust boundaries ni dependencias nuevas (los nombres de artefactos no son superficie de seguridad).
- [ ] **PERFORMANCE** — no aplica: sin código ni hot paths.

## Steps

### Step 1: Evidence pack de los 9 artefactos + numeración
- **Archivos:** `ADR-030`, `docs/dev/Backlog.md` (L939), `docs/dev/architecture/adr/`
- **Acción:** consolidar tabla de los 9 artefactos (nombre actual, registry/estado, publish flag, precedente) + verificar siguiente número de ADR libre.
- **Verify:** notas con tabla de 9 filas con fuente; `Get-ChildItem docs/dev/architecture/adr/` confirma `ADR-045` libre (o ajustar número).
- **Estado:** ⬜ PENDING

### Step 2: Draft del ADR (evidencia + skeleton, status `proposed`)
- **Archivos:** `docs/dev/architecture/adr/ADR-045-naming-freeze.md` (nuevo)
- **Acción:** crear el ADR con template (`Context`/`Decision`/`Consequences`), tablas de evidencia y política de alias; marcar explícitamente las secciones que **articula el owner** (Regla 5). No inventar decisiones.
- **Verify:** archivo existe; `rg -n "^## (Context|Decision|Consequences)" docs/dev/architecture/adr/ADR-045-naming-freeze.md` (3 hits); `status: proposed`.
- **Estado:** ⬜ PENDING

### Step 3: Gate P — articulación y firma del owner
- **Archivos:** `ADR-045` (status), `docs/dev/tasks/DEF-04.md` (registro)
- **Acción:** `question` al owner con: tabla 9 artefactos, política de alias, relación con ADR-030 y decisiones pendientes referenciadas; con la firma, owner articula y el status pasa a `accepted`.
- **Verify:** articulación/firma registrada en Notas; **sin firma → queda `proposed`** (STOP; no marcar accepted).
- **Estado:** ⬜ PENDING

### Step 4: Referencia desde VERSIONING.md + coordinación HARD-01 + gates
- **Archivos:** `docs/api/VERSIONING.md` (referencia), coordinación `DEPRECATIONS.md` (HARD-01)
- **Acción:** agregar la referencia al ADR en `VERSIONING.md` (§Deprecation policy / surfaces); verificar/anotar el cross-link a `DEPRECATIONS.md`; correr gates y commit local `docs:`.
- **Verify:** `rg -n "ADR-045|naming freeze" docs/api/VERSIONING.md`; link `DEPRECATIONS.md` resuelto o nota de coordinación registrada; `pwsh scripts/validate-docs-coverage.ps1` exit 0.
- **Estado:** ⬜ PENDING

## Dependencias
- **HARD-01** (coordina `DEPRECATIONS.md` — mismo F0; no bloquea el draft del ADR; bloquea el link final si el archivo aún no existe → coordinar orden).
- **LEG-01** (owner, informativo — dictamen de marca; no bloquea el freeze de nombres de artefactos).
- Base: API-01..09 ✅ (último breaking pre-lanzamiento — congelar ahora es gratis).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` (leaf, no implementa) — fallback: `doubt-driven-development` en contexto fresco.
- **Enfoque:** ¿el ADR lo articula/firma el owner (Regla 5) y no la IA? ¿Los 9 artefactos están completos (sin olvidos)? ¿La política de alias tiene fechas de remoción y nunca rename silencioso? ¿Cero renames ejecutados?
- **Cómo se probó:** comandos del contrato ejecutados y registrados (no auto-reporte) + `git diff --name-only` acotado + OCR delegation sin Critical/High.
- **Checklist anti-hábitos tóxicos** (contrato de comportamiento — el revisor verifica que el implementador NO haya incurrido en ninguno antes de aprobar; fuente §12 de `docs/Investigaciones/2026-08-10-agent-engineering/agent-02-task-execution.md`):
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
- **Veredicto:** ⬜ pendiente (✅ approve | ❌ cambios requeridos)

## Notas
- **No aplica Fase 1 — Evidencia de Debugging** (tipo Docs, no Bug).
- **Scope discipline:** no tocar READMEs/CHANGELOG/README_ES (referencias entrantes intactas); no renombrar nada; no resolver las decisiones pendientes de ADR-030 (dominio/PyPI/npm) — quedan referenciadas.
- **Release N/A:** es policy/ADR pre-release; el freeze se materializa vía release-plz (0.8.0 / 1.0) sin cambios manuales de versión.
- **Regla 5:** forcing function — el owner articula; la IA solo aporta evidencia (tablas, registries, riesgos).
- Creado por `vanta-docs` (subagente) desde el master roadmap — estado inicial ⬜ PENDING.
