# HARD-04: vanta-memory — spec de fachada + triggers de exposición

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 4)
- **Fuente:** master Task 4 · Backlog `FIND-160` (co-referencia, L258) · `docs/dev/tasks/API-STD-15.md` (Gate P core-only)
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🟡
- **Tipo:** Docs
- **Turns estimados:** 6
- **Creado:** 2026-09-26T19:55
- **last-synced:** 2026-09-26T19:55
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 4 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-mcp` (in-process) · `vanta-proxy` (`memory_tools.rs` TOOL_CAPTURE/TOOL_SEARCH, `capture.rs`, `inject.rs`) · `desktop/src-tauri` (`commands/memory.rs`) · tests `vanta-memory/tests/l0_capture.rs` |
| Callees | `vantadb` core (`Embedded`, storage, text index, HNSW) · módulos `vanta-memory` (core, utils, services, adapters, offload, context_engine, gateway, seed, ingest — `lib.rs:37-61`) |
| Implicaciones | Doc-only: no cambia API ni código · 0 símbolos en bindings (Gate P) · `validate-docs-coverage` NO escanea `vanta-memory` (página = referencia manual) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/api/VANTA_MEMORY.md` (181L) · `vanta-memory/src/lib.rs` (66L) · `docs/dev/tasks/API-STD-15.md` · `docs/dev/Backlog.md` (fila `FIND-160`, L258)
- **Archivos referenciados hacia dentro (imports/dependencias):** `vanta-memory/src/lib.rs:26` cita `docs/api/VANTA_MEMORY.md` como superficie estable; la crate depende de `vantadb::sdk::Embedded`
- **Archivos que referencian a los editados (referencias entrantes):** `docs/api/VERSIONING.md:48` (superficie #11 → `VANTA_MEMORY.md`) · ADR-029 (cita la página) · doc comment en `vanta-memory/src/lib.rs:26`
- **Veredicto impacto:** **bajo** — solo se agregan 2 secciones; ningún consumidor compila contra el doc; sin código ni bindings.

## Contrato
"`docs/api/VANTA_MEMORY.md` incluye §Facade (capture/recall/seed/ingest, firma conceptual + degradación) Y §Exposure triggers T1–T4 (≥5 pedidos externos / adapter ICP-03 bloqueado / 2 releases sin breaking / stranger-tests confirman demanda local-first) Y `pwsh scripts/validate-docs-coverage.ps1` verde Y `rg \"vanta[_-]memory\" vantadb-python vantadb-ts vantadb-node vantadb-wasm` = 0 matches"

## Spec (SDD — obligatoria: la task documenta superficie nueva)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Presentación de la facade | A) firmas conceptuales (pseudo-Rust) + tabla de degradación por capa, etiquetadas `candidate, not published` / B) solo paths de módulo | A | ✅ decidido-por-evidencia (master Task 4 pre-mortem F1: "facade como contrato firme → marcar candidate") |
| 2 | Triggers T1–T4 | A) medibles: T1 ≥5 pedidos externos · T2 adapter ICP-03 bloqueado · T3 2 releases sin breaking · T4 stranger-tests (FASE-A) confirman demanda local-first / B) cualitativos | A | ✅ decidido-por-evidencia (master Task 4 contrato + pre-mortem F2 "fechas/umbrales numéricos") |
| 3 | ¿Cubrir FIND-160 (dream/gateway/ingest/services) acá? | A) NO — co-referencia; se paga en HARD-06 / B) incluir ahora | A | ✅ decidido-por-evidencia (master Task 4 Notas: "FIND-160 se paga en HARD-06") |
| 4 | ¿Proponer símbolos en bindings? | A) PROHIBIDO (Gate P core-only) / B) proponer exposición | A | ✅ decidido-por-evidencia (`API-STD-15` §2: "core-only + API Rust estable; exponer post-release con demanda") |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) `vanta-memory` sigue **core-only** (`publish = false`, 0 símbolos en bindings); (2) la facade queda etiquetada **`candidate, not published`** — no es contrato publicado; (3) `VANTA_MEMORY.md` sigue siendo referencia manual (no prometer gate automático de `validate-docs-coverage` sobre `vanta-memory`).
- **Comandos de verificación:** `rg "vanta[_-]memory" vantadb-python vantadb-ts vantadb-node vantadb-wasm` → 0 matches (exit 1) · `pwsh scripts/validate-docs-coverage.ps1` → exit 0.
- **Deuda pendiente:** `FIND-160` (módulos sin doc: `core/dream/`, `gateway/approval_handlers`, `ingest/auto_sync`, `services/conversation_hook`, `core/memory_generation_log`) — se paga en HARD-06.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|---|---|
| `activeGoal` | `# HARD-04: vanta-memory — spec de fachada + triggers de exposición` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Step 1 ⬜ PENDING (evidence pack + baseline `rg` bindings) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia (comandos ejecutados) |
| `nextTask` | HARD-05 |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (task doc-only; no introduce deuda nueva).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable ✅ (§Facade + §Exposure triggers + 0 bindings + validate-docs-coverage verde) + markdownlint sin errores |
| **Commit** | 1 commit atómico `docs:` (~doc-only), `git diff` limpio, verificación mecánica registrada (nunca auto-reporte) |
| **Release** | N/A justificado (doc de crate `publish=false`; sin artefacto de release) — ver Notas |

## Herramientas necesarias
- `codegraph_explore` (blast radius — ejecutado en definición: callers `vanta-proxy`/`desktop`/tests)
- `pwsh scripts/validate-docs-coverage.ps1` · `npx markdownlint-cli2 docs/api/VANTA_MEMORY.md`
- `pwsh dev-tools/ocr-review.ps1` (input del review P2-01)

**Skills cargadas (SDP):** `SDP: campaign_discover_skills_v2 phase=BUILD → campaign-executor, documentation-and-adrs, api-and-interface-design, spec-driven-development, writing-guidelines, doubt-driven-development`
- `campaign-executor` — flujo pipeline/task system (base).
- `documentation-and-adrs` — estándar de docs técnicas/ADR.
- `api-and-interface-design` — facade como candidato de superficie (boundaries).
- `spec-driven-development` — tabla Spec.
- `writing-guidelines` — docs técnicas en inglés.
- `doubt-driven-development` — evitar prometer superficie no estable (pre-mortem F1).

## Investigation Notes
- Gate P (2026-09-24, `API-STD-15` §1-2): 4/4 recomendadas aprobadas — **core-only memory** ("exponer post-release con demanda", D42/D43).
- Crate: `publish = false`, 423 ítems pub, 9 módulos (`vanta-memory/src/lib.rs:37-61`: core, utils, services, adapters, offload, context_engine, gateway, seed, ingest).
- Consumidores in-process: `vanta-proxy` (`memory_tools.rs:74` execute capture/search; `capture.rs`; `inject.rs`; `server.rs`), `desktop/src-tauri/src/commands/memory.rs`, `vantadb-mcp`.
- Superficie de facade a documentar (firma conceptual): `AutoCaptureHook::capture` (`core/hooks/auto_capture.rs:84`) · `perform_auto_recall` + `RecallConfig`/`RecallScope` (`core/hooks/auto_recall.rs`) · `seed` (módulo + `vanta-seed` CLI, `VANTA_MEMORY.md:148-149`) · `ingest::worker::run`/`run_with_progress` (`VANTA_MEMORY.md:125-131`).
- Degradación P4 ya documentada (`VANTA_MEMORY.md:48-67`) — §Facade debe enlazarla, no duplicarla.
- `FIND-160` (L258): cobertura incompleta post-MEM-38 — contrato: sección/tabla por módulo o exclusión motivada. **No es de esta task** (HARD-06).
- Evidencia 0 bindings (2026-09-26): `rg "vanta[_-]memory" vantadb-python vantadb-ts vantadb-node vantadb-wasm` → 0 matches (exit 1).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — decisiones de presentación resueltas por evidencia (Spec) |
| Pendientes de ejecución (downhill) | 4 — Steps 1–4 |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — no aplica: doc-only, no toca trust boundaries ni input de usuario; sin dependencias nuevas (la degradación P4 que documenta ya está implementada y testada).
- [ ] **PERFORMANCE** — no aplica: no toca hot paths (sin código).

## Steps

### Step 1: Evidence pack + baseline de bindings
- **Archivos:** `docs/api/VANTA_MEMORY.md`, `vanta-memory/src/lib.rs`, `docs/dev/tasks/API-STD-15.md`, `docs/dev/Backlog.md`
- **Acción:** confirmar baseline `rg` de 0 símbolos en bindings y colectar la superficie real de facade (capture/recall/seed/ingest) con refs file:line.
- **Verify:** `rg "vanta[_-]memory" vantadb-python vantadb-ts vantadb-node vantadb-wasm` → 0 matches (exit 1) + notas con las 4 operaciones citadas.
- **Estado:** ⬜ PENDING

### Step 2: Draft §Facade (firmas conceptuales + degradación)
- **Archivos:** borrador en este task file (sección temporal) — sin editar aún `docs/api/`
- **Acción:** redactar §Facade: capture/recall/seed/ingest con firma conceptual, enlace a la tabla de degradación P4 existente y etiqueta `candidate, not published`. PROHIBIDO proponer símbolos nuevos en bindings.
- **Verify:** checklist 4/4 operaciones con ref file:line; `rg -n "candidate, not published" docs/dev/tasks/HARD-04.md`.
- **Estado:** ⬜ PENDING

### Step 3: Editar `docs/api/VANTA_MEMORY.md` (§Facade + §Exposure triggers T1–T4)
- **Archivos:** `docs/api/VANTA_MEMORY.md`
- **Acción:** integrar §Facade y §Exposure triggers T1–T4 (tabla medible: umbral, fuente de medición, dueño) después de §Scope & stability; eliminar el borrador del task file.
- **Verify:** `rg -n "## Facade|## Exposure triggers" docs/api/VANTA_MEMORY.md` (2 hits) + `npx markdownlint-cli2 docs/api/VANTA_MEMORY.md`.
- **Estado:** ⬜ PENDING

### Step 4: Gates mecánicos + cierre
- **Archivos:** `docs/dev/tasks/HARD-04.md` (estado/recitation)
- **Acción:** correr gates y registrar evidencia (comando + resultado) en el task file; commit local `docs:`.
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` exit 0 · `pwsh dev-tools/ocr-review.ps1` sin Critical/High · recitation actualizada.
- **Estado:** ⬜ PENDING

## Dependencias
- **Ninguna dura.** Co-referencias: `API-STD-15` ✅ (fuente Gate P) · `FIND-160` → HARD-06 (no bloquea; no cubrir módulos acá).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` (leaf, no implementa) — fallback: `doubt-driven-development` en contexto fresco.
- **Enfoque:** ¿la facade quedó como `candidate, not published` (sin promesa de estabilidad)? ¿Los triggers T1–T4 son medibles con fuente? ¿Cero señales de exposición en bindings?
- **Cómo se probó:** comandos del contrato ejecutados y registrados (no auto-reporte) + diff solo-docs + OCR delegation sin Critical/High.
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
- **Scope discipline:** no tocar bindings, `vanta-memory/src/**`, ni `VERSIONING.md` (superficie #11 ya lista `vanta-memory`); FIND-160 no se cierra acá.
- **Release N/A:** la página es referencia de un crate `publish=false`; el cambio no participa del contrato de release.
- Creado por `vanta-docs` (subagente) desde el master roadmap — estado inicial ⬜ PENDING.
