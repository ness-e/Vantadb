---
title: "DEF-04: Naming freeze 0.7.0→1.0 (ADR + política de alias)"
kind: task
description: "ADR con los 9 artefactos congelados (vantadb, vantadb-py, vantadb-node, vantadb-ts, vantadb-server, vantadb-mcp, vanta-cli, vanta-proxy, vanta-memory) + política de alias/deprecación (alias con fecha de remoción, nunca rename..."
---

# DEF-04: Naming freeze 0.7.0→1.0 (ADR + política de alias)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 12)
- **Fuente:** Backlog `DEF-04` (L939) · master Task 12 · R1 (congelar pre-usuarios es gratis)
- **Esfuerzo:** 🟢 1d
- **Prioridad:** 🔴
- **Tipo:** Docs (ADR + política)
- **Turns estimados:** 8
- **Creado:** 2026-09-26T19:55
- **last-synced:** 2026-09-27T00:00
- **Estado:** ✅ COMPLETED (2026-09-27 — firma owner Gate P ✅; review fresco P2-01 ✅; commit LEAD)
- **Incógnitas (uphill):** 0 — firma owner completada (Gate P, 2026-09-27)
- **Pendientes (downhill):** 0 — 4/4 ✅ (Step 3 firmado)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/api/VERSIONING.md` (referencia desde §Deprecation policy) · `docs/api/DEPRECATIONS.md` (HARD-01 — política de alias) · READMEs que citan ADR-0030 (`README.md:80`, `README_ES.md:79`, `docs/api/PYTHON_SDK.md:79`, `vantadb-ts/README.md:163`) · LEG-01 (owner — dictamen de marca) · MKT-18f (publicación adapters con nombres congelados) |
| Callees | `ADR-0030` (evidencia de marca) · `ADR-0041`/`ADR-0047`/`ADR-0042` (precedente de renames/anti-stutter) · `docs/dev/_templates/adr.md` · `VERSIONING.md` §Deprecation policy |
| Implicaciones | Doc-only; **cero renames** (ADR-0030 regla práctica) · no cambia nada publicado (crates.io `vantadb`, PyPI `vantadb-py`, npm `vantadb`) · ADR queda `proposed` hasta firma owner |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/dev/architecture/adr/ADR-0030-brand-identity-naming-convention.md` (104L) · `docs/api/VERSIONING.md` (102L) · `docs/dev/_templates/adr.md` · `docs/dev/Backlog.md` (fila `DEF-04`, L939)
- **Archivos referenciados hacia dentro (imports/dependencias):** ADR-0030 referencia `docs/dev/_templates/adr.md` (template) y estilo ADR-0027/030 (`ADR-0031:205`); DEF-04 depende del template para el draft
- **Archivos que referencian a los editados (referencias entrantes):** ADR-0030 citada por `README.md:80`, `README_ES.md:79`, `docs/api/PYTHON_SDK.md:79`, `vantadb-ts/README.md:163`, `CHANGELOG.md:1925`, `ADR-0031:205` — **no se editan** (solo el ADR nuevo las referencia)
- **Veredicto impacto:** **bajo** — 1 archivo nuevo (ADR) + 1 referencia en `VERSIONING.md`; sin código; `DEPRECATIONS.md` no existe aún (lo crea HARD-01 — solo coordinar).

## Contrato
"ADR con los 9 artefactos congelados (vantadb, vantadb-py, vantadb-node, vantadb-ts, vantadb-server, vantadb-mcp, vanta-cli, vanta-proxy, vanta-memory) + política de alias/deprecación (alias con fecha de remoción, nunca rename silencioso) Y firma owner (Regla 5) Y referencia desde VERSIONING.md"

## Spec (SDD — decisión con tradeoffs)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Numeración/formato del ADR | A) `ADR-0045-naming-freeze.md` (siguiente libre tras ADR-0044; convención `ADR-NNN-slug`) + template `_templates/adr.md` / B) esquema legacy `NNN_titulo.md` | A | ✅ decidido-por-evidencia (verificar siguiente libre al ejecutar; convención existente) |
| 2 | Relación con ADR-0030 (`proposed`) | A) ADR-0045 referencia/extiende ADR-0030 (ADR-0030 = convención de marca; ADR-0045 = freeze 0.7→1.0 + alias) / B) ADR-0045 supersede ADR-0030 | A (owner confirma) | ✅ owner confirmó (Gate P 2026-09-27 — "Firmo: apruebo D1–D3") |
| 3 | Política de alias | A) alias siempre con fecha de remoción + ventana ≥1 MINOR (VERSIONING §Deprecation policy) + nunca rename silencioso / B) sin política formal | A | ✅ decidido-por-evidencia (master Task 12 contrato + `VERSIONING.md:89-94`) |
| 4 | Rol IA vs owner (Regla 5) | A) IA: evidencia + draft; owner: articula Context/Decision/Consequences + firma / B) IA redacta completo y auto-accept | A | ✅ decidido-por-evidencia (AGENTS Regla 5 + master pre-mortem F1) |
| 5 | Decisiones ADR-0030 pendientes (dominio, PyPI ownership, `vantadb-node` publish, case repo) | A) referenciar (no resolver acá; LEG-01 owner) / B) resolver en este ADR | A | ✅ decidido-por-evidencia (ADR-0030 §Decisiones pendientes; LEG-01 en carril owner) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) **cero renames** de artefactos publicados (ADR-0030 regla práctica); (2) ADR `proposed` hasta firma del owner — la IA nunca lo marca `accepted`; (3) todo alias con fecha de remoción; (4) no editar READMEs/CHANGELOG (solo referenciar); `DEPRECATIONS.md` es de HARD-01.
- **Comandos de verificación:** `rg -n "ADR-0045" docs/api/VERSIONING.md` (referencia presente) · `git diff --name-only` = solo ADR nuevo + `VERSIONING.md` (+ task file) · `pwsh scripts/validate-docs-coverage.ps1` → exit 0.
- **Deuda pendiente:** cross-link a `DEPRECATIONS.md` (coordinación HARD-01 — el archivo no existe hasta que HARD-01 lo cree).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|---|---|
| `activeGoal` | `# DEF-04: Naming freeze 0.7.0→1.0 (ADR + política de alias)` |
| `lastAction` | Firma owner (Gate P) + review fresco APPROVE |
| `result` | OK |
| `nextAction` | LEAD: commit local `docs:` (ADR-0045 + VERSIONING + task file) |
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

**Skills cargadas (SDP v3, pre-run 2026-09-27):** `campaign-executor` · `progreso` · `documentation-and-adrs` · `api-and-interface-design` · `writing-guidelines` · `writing-plans` · `incremental-implementation` · `test-driven-development` · **PINNED (policy):** `documentation-and-adrs`, `api-and-interface-design`
- `campaign-executor` — flujo pipeline/task system (base).
- `documentation-and-adrs` — estándar ADR + template.
- `deprecation-and-migration` — política de alias/ventana de remoción (keyword mapping).
- `api-and-interface-design` — congelar nombres = contrato de superficies.
- `spec-driven-development` — tabla Spec.
- `writing-guidelines` — prosa técnica.

## Investigation Notes
- **9 artefactos a congelar (evidence pack ✅ 2026-09-27 — file:line + registry live):**

| # | Nombre congelado | Qué nombra | Registry / estado (live 2026-09-27) | Evidencia |
|---|---|---|---|---|
| 1 | `vantadb` | Crate Rust core | crates.io **0.7.0** ✅ | `Cargo.toml:2` |
| 2 | `vantadb-py` | Distribución Python (PyPI) | PyPI **0.7.0** ✅ | `vantadb-python/pyproject.toml:6` |
| 3 | `vantadb-node` | Bindings nativos napi-rs | npm **404 — nunca publicado** | `vantadb-node/package.json:2-3` |
| 4 | `vantadb-ts` | Dir/repo del SDK TypeScript (npm publica como `vantadb`) | dir ✅ + npm `vantadb` **0.7.0** ✅ | `vantadb-ts/package.json:2` |
| 5 | `vantadb-server` | Crate HTTP server | workspace · `publish=false` | `vantadb-server/Cargo.toml:2,6` |
| 6 | `vantadb-mcp` | Crate MCP server | workspace · `publish=false` | `vantadb-mcp/Cargo.toml:2,7` |
| 7 | `vanta-cli` | Binario CLI | `[[bin]]` del crate core | `Cargo.toml:350-352` |
| 8 | `vanta-proxy` | Crate LLM proxy | workspace · `publish=false` | `vanta-proxy/Cargo.toml:2,6` |
| 9 | `vanta-memory` | Crate memoria L0–L3 | workspace · `publish=false` | `vanta-memory/Cargo.toml:2,7` |

  (master Task 12 + Backlog `DEF-04` fila L946 — drift: el task file citaba L939.)
- **ADR-0030 (status `proposed`):** tabla de 11 superficies/artefactos con evidencia live 2026-08-25 (crates.io/PyPI/npm/GitHub/DNS) + 5 decisiones pendientes del owner (dominio, PyPI ownership `DevpNess`→`ness-e`, publicar `vantadb-node`, case del repo, metadata PyPI). Regla práctica: "no renames; cada ecosistema conserva el nombre publicado".
- **Numeración (verificado 2026-09-27):** máximo = `ADR-0044`; **`ADR-0045` libre** (`Get-ChildItem *045*` → vacío) → archivo `ADR-0045-naming-freeze.md` (convención `ADR-NNN-slug` + template `docs/dev/_templates/adr.md`).
- **Precedente:** ADR-0041 (error variant renames) y ADR-0042 (pub signatures major) — cómo el repo trató renames pre-0.7; API-01..09 completadas = último breaking pre-lanzamiento (base lista).
- **VERSIONING.md (re-verificado 2026-09-27):** §Deprecation policy: deprecación ≥1 MINOR + removal con `feat!:`/`BREAKING CHANGE:`; §Enforcement: `validate-docs-coverage` + tests. **`docs/api/DEPRECATIONS.md` YA EXISTE** (HARD-01 avanzó; `last_reviewed: 2026-09-27`; registra instancias y linkea a VERSIONING §Deprecation policy) → el cross-link es ejecutable ahora, sin esperar a HARD-01.
- **Coordinaciones:** HARD-01 (mismo F0; `DEPRECATIONS.md` + `COMPATIBILITY.md`) · LEG-01 (owner — dictamen de marca informa display/repo, no bloquea el freeze de nombres de artefactos) · HIG-02 (firma waiver ADR-0047 — carril owner, contexto).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — firma owner completada (Gate P 2026-09-27) |
| Pendientes de ejecución (downhill) | 0 — 4/4 ✅ |
| % completado | 100% (firma ✅; review fresco ✅; commit LEAD) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — no aplica: doc-only, sin trust boundaries ni dependencias nuevas (los nombres de artefactos no son superficie de seguridad).
- [ ] **PERFORMANCE** — no aplica: sin código ni hot paths.

## Steps

### Step 1: Evidence pack de los 9 artefactos + numeración
- **Archivos:** `ADR-0030`, `docs/dev/Backlog.md` (L946 — drift: la fila real es L946, no L939), `docs/dev/architecture/adr/`
- **Acción:** consolidar tabla de los 9 artefactos (nombre actual, registry/estado, publish flag, precedente) + verificar siguiente número de ADR libre.
- **Verify:** ✅ 2026-09-27 — tabla 9/9 en §Investigation Notes con file:line; `ADR-0045` libre; registry live: crates.io `vantadb` 0.7.0 · PyPI `vantadb-py` 0.7.0 · npm `vantadb` 0.7.0 · npm `vantadb-node` HTTP 404.
- **Estado:** ✅ DONE (2026-09-27)

### Step 2: Draft del ADR (evidencia + skeleton, status `proposed`)
- **Archivos:** `docs/dev/architecture/adr/ADR-0045-naming-freeze.md` (nuevo)
- **Acción:** crear el ADR con template (`Context`/`Decision`/`Consequences`), tablas de evidencia y política de alias; marcar explícitamente las secciones que **articula el owner** (Regla 5). No inventar decisiones.
- **Verify:** ✅ 2026-09-27 — archivo creado; headers `Context`/`Decision`/`Consequences` = 3 hits (hoy L17/L60/L115); `status: proposed` (al crear; hoy `accepted` por firma); tabla 9/9 filas (`rg -c '^\| [0-9] \| '` = 9); 3 marcas `[OWNER]` (hoy confirmadas); §Owner sign-off firmado.
- **Estado:** ✅ DONE (2026-09-27)

### Step 3: Gate P — articulación y firma del owner
- **Archivos:** `ADR-0045` (status), `docs/dev/tasks/DEF-04.md` (registro)
- **Acción:** `question` al owner con: tabla 9 artefactos, política de alias, relación con ADR-0030 y decisiones pendientes referenciadas; con la firma, owner articula y el status pasa a `accepted`.
- **Verify:** ✅ 2026-09-27 — Gate P ejecutado por el LEAD (`question` al owner): **"Firmo: apruebo D1–D3"**. `status` → `accepted`; §Owner sign-off completado (articulación + firma Eros 2026-09-27 + riesgos aceptados); banner del ADR actualizado.
- **Estado:** ✅ DONE (2026-09-27 — firmado por el owner)

### Step 4: Referencia desde VERSIONING.md + coordinación HARD-01 + gates
- **Archivos:** `docs/api/VERSIONING.md` (referencia), coordinación `DEPRECATIONS.md` (HARD-01)
- **Acción:** agregar la referencia al ADR en `VERSIONING.md` (§Deprecation policy / surfaces); verificar/anotar el cross-link a `DEPRECATIONS.md`; correr gates y commit local `docs:`.
- **Verify:** ✅ 2026-09-27 — `rg -n "ADR-0045|naming freeze" docs/api/VERSIONING.md` → L108-109; `DEPRECATIONS.md` ya existe (HARD-01) y el cross-link resuelve (ADR-0045 + VERSIONING lo referencian); `pwsh scripts/validate-docs-coverage.ps1` → exit 0 · 0 gaps (registrado vía `campaign_verify_cmd`). **Commit local: NO ejecutado — instrucción del orquestador (wave F1b): el LEAD commitea** el set atómico (ver RESULTADO).
- **Estado:** ✅ DONE (2026-09-27) — commit delegado al LEAD

## Dependencias
- **HARD-01** (coordina `DEPRECATIONS.md` — mismo F0; ✅ el archivo ya existe al 2026-09-27 → cross-link ejecutado; sin bloqueo pendiente).
- **LEG-01** (owner, informativo — dictamen de marca; no bloquea el freeze de nombres de artefactos).
- Base: API-01..09 ✅ (último breaking pre-lanzamiento — congelar ahora es gratis).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` (leaf, no implementa) — fallback: `doubt-driven-development` en contexto fresco.
- **Enfoque:** ¿el ADR lo articula/firma el owner (Regla 5) y no la IA? ¿Los 9 artefactos están completos (sin olvidos)? ¿La política de alias tiene fechas de remoción y nunca rename silencioso? ¿Cero renames ejecutados?
- **Cómo se probó:** comandos del contrato ejecutados vía `campaign_verify_cmd` (no auto-reporte): `rg "ADR-0045|naming freeze"` ✅ · headers ADR ✅ · `validate-docs-coverage` exit 0 ✅; `git diff --name-only` revisado (propios: ADR-0045 + `VERSIONING.md` + task file; hay cambios ajenos de sesiones paralelas F1b en el working tree — no tocados); OCR `pwsh dev-tools/ocr-review.ps1 -Format json` → sin findings (`.md` = `unsupported_ext`; advisory limpio).
- **Ejecución P2-01 (2026-09-27):** ⚠️ **DEGRADADO** — esta sesión no dispone de tool de sub-agentes (no se pudo spawnear `vanta-review` fresco); se aplicó `doubt-driven-development` degradado (auto-cuestionamiento con separador, sin contexto fresco real) que detectó y corrigió 5 imprecisiones antes de cerrar. **Escalado al LEAD:** review fresco requerido antes del ACCEPT (bloqueado además por firma owner, Regla 5).
- **Review fresco P2-01 (LEAD, 2026-09-27):** `vanta-review` sesión `ses_f1b7cf628ffeY30trC2X4LpjIw` (≠ autor y ≠ review degradado) → **✅ APPROVE**. Evidencia: contrato re-ejecutado (`rg ADR-0045` → VERSIONING L109 · headers 3 hits · coverage exit 0) · 9/9 refs file:line exactas + 4/4 registries live (crates.io/PyPI/npm 0.7.0; npm vantadb-node 404) · Regla 5 verificada (sin auto-aceptación IA) · D2 coherente con VERSIONING+DEPRECATIONS · `git diff -M` sin renames · numeración libre (046 vacío).
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
- **Veredicto:** ✅ **APPROVE** — review fresco `vanta-review` (sesión `ses_f1b7cf628ffeY30trC2X4LpjIw`, 2026-09-27). Optional: link directo DEPRECATIONS→ADR-0045 en la próxima pasada de HARD-01. ACCEPT habilitado (payload review fresh para HARD-07).

## Notas
- **No aplica Fase 1 — Evidencia de Debugging** (tipo Docs, no Bug).
- **Scope discipline:** no tocar READMEs/CHANGELOG/README_ES (referencias entrantes intactas); no renombrar nada; no resolver las decisiones pendientes de ADR-0030 (dominio/PyPI/npm) — quedan referenciadas.
- **Release N/A:** es policy/ADR pre-release; el freeze se materializa vía release-plz (0.8.0 / 1.0) sin cambios manuales de versión.
- **Regla 5:** forcing function — el owner articula; la IA solo aporta evidencia (tablas, registries, riesgos).
- Creado por `vanta-docs` (subagente) desde el master roadmap — estado inicial ⬜ PENDING.
- **Step 3 — BLOQUEO por firma owner (Regla 5), 2026-09-27:** `ADR-0045-naming-freeze.md` está completo (evidence pack 9/9 + borrador D1–D3 + secciones `[OWNER]` + §Owner sign-off) con `status: proposed`. La IA no firma ni auto-acepta. **El LEAD eleva `question` al owner** con: (a) tabla 9/9 con registry live; (b) política D1–D3 (freeze `0.7.0→1.0` + alias con fecha de remoción + nunca rename silencioso); (c) relación con ADR-0030 (complementario; 5 decisiones pendientes referenciadas, sin resolver); (d) stop condition. Con firma → editar `status: accepted` + completar §Owner sign-off (articulación + nombre/fecha) + registrar en Notas. Sin firma → queda `proposed` y la tarea cierra con este bloqueo (🟡 INCOMPLETO), sin marcar `accepted`.
- **Firma Gate P (2026-09-27):** el LEAD elevó `question` al owner (tabla 9/9 + D1–D3 + relación ADR-0030); respuesta: **"Firmo: apruebo D1–D3"**. Registro: ADR-0045 §Owner sign-off (firma Eros, 2026-09-27) + este task file. Trazabilidad: el `question` no deja rastro en `campaign_get_task_detail`/`memory_recall`; fuente repo = §Owner sign-off.
