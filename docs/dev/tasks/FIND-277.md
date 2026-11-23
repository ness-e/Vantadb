# FIND-277: Filtrar registros con `superseded_by` activo en lecturas de recall de `l1_reader.rs`

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md`
- **Fuente:** `docs/dev/Backlog.md` (Hallazgos pendientes L271) / Plan Task 23
- **Esfuerzo:** 🟡 2h
- **Prioridad:** 🟠
- **Tipo:** Rust
- **Turns estimados:** 15
- **Creado:** 2026-11-18T12:00
- **last-synced:** 2026-11-18
- **Estado:** ✅ COMPLETED
- **Commits:** `6b5170c19`, `cc1ec126d`, `93fae5b5f`
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-memory::core::hooks::auto_recall`, `vanta-memory::pipeline_worker` (consolidación L2) |
| Callees | `vanta-memory::core::record::l1_reader` |
| Implicaciones | Las rutas de recall activo y consolidación L2 filtran registros con `superseded_by` no vacío; los lectores crudos (`read_namespace_records`, `read_session_records`) se conservan intactos para `l1_writer`, auditoría, decay y cascade-forget |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vanta-memory/src/core/record/l1_reader.rs`, `vanta-memory/src/core/hooks/auto_recall.rs`, `vanta-memory/src/pipeline_worker.rs`
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `vanta-memory::core::record::L1Record`, `vantadb::Embedded`
- **Archivos que referencian a los editados (referencias entrantes):** `vanta-memory/src/core/hooks/auto_recall.rs`, `vanta-memory/src/pipeline_worker.rs`, `vanta-memory/src/core/record/l1_writer.rs`
- **Veredicto impacto:** medio — separa explícitamente los lectores activos (`read_active_*` + predicado `is_active_record`) de los lectores crudos para no romper escrituras ni auditoría histórica.

## Contrato
`cargo nextest run --profile audit -p vanta-memory -E "test(supersed) or test(l1_reader) or test(recall)" --locked --build-jobs 2` pasa filtrando todo registro con `superseded_by` no vacío en las rutas de recall y L2.

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Separación de lectores activos vs crudos | A: Añadir `read_active_namespace_records` / `read_active_session_records` + helper `is_active_record` dejando `read_namespace_records` crudo para writer/audit / B: Filtrar incondicionalmente en `read_namespace_records` (rompe writer/audit/decay) | A | ✅ decidido-por-evidencia (ref: `vanta-memory/src/core/record/l1_reader.rs:58-90,118-135`) |
| 2 | Semántica de `superseded_by` vacío o whitespace | A: `superseded_by.as_deref().is_some_and(|s| !s.trim().is_empty())` (blank = activo) / B: `is_some()` estricto | A | ✅ decidido-por-evidencia (ref: `vanta-memory/src/core/record/l1_reader.rs:58-90`, commit `93fae5b5f`) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** Ninguna ruta de recall activo (`auto_recall.rs:477,503,677,763,1036,1048`) ni de consolidación L2 (`pipeline_worker.rs:643`) debe servir registros cuyo `superseded_by` tenga contenido no vacío; las rutas de escritura/auditoría/decay deben seguir viendo todos los registros vía lectores crudos.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vanta-memory -E "test(supersed) or test(l1_reader) or test(recall)" --locked --build-jobs 2`
- **Deuda pendiente:** `FIND-659` (`H-NEW-2` en `reflection/mod.rs:356`, seguimiento separado en Backlog para usar `read_active_session_records` en reflexión L2).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | `# FIND-277: Filtrar registros con superseded_by activo en lecturas de recall de l1_reader.rs` |
| `lastAction` | Step 3 ✅ — `recall_candidates` unificado con `is_active_record` y test de ruta ADR en `93fae5b5f` |
| `result` | `OK` ↔ ✅ COMPLETED |
| `nextAction` | Ninguna (tarea completada) |
| `contract` | `cargo nextest run --profile audit -p vanta-memory -E "test(supersed) or test(l1_reader) or test(recall)" --locked --build-jobs 2` → PASS |
| `nextTask` | `FIND-278` |

```yaml
contract:
  verificacion: "cargo nextest run --profile audit -p vanta-memory -E \"test(supersed) or test(l1_reader) or test(recall)\" --locked --build-jobs 2 -> PASS (53/53 recall/provenance, 727/727 suite)"
  evidencia:
    - claim: "read_active_* + is_active_record cubren las 5 rutas de recall, L2 consolidation y recall_candidates"
      evidencia: "vanta-memory/src/core/record/l1_reader.rs:58-90,118-135,399; vanta-memory/src/core/hooks/auto_recall.rs:477,503,677,763,1036,1048; vanta-memory/src/pipeline_worker.rs:643"
      confianza: alta
  artefactos:
    - "vanta-memory/src/core/record/l1_reader.rs"
    - "vanta-memory/src/core/hooks/auto_recall.rs"
    - "vanta-memory/src/pipeline_worker.rs"
  invariantes: "Recall activo y L2 nunca inyectan registros superseded; raw readers intactos para writer/audit/decay"
  deuda: "FIND-659 (hermano en reflection/mod.rs:356 registrado en Backlog)"
  queda_pendiente: "ninguno"
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (cierra brecha F7 de MEMG-01 unificando el predicado `is_active_record`).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato mecánico cumplido + `cargo fmt --check` + `cargo clippy -p vanta-memory -- -D warnings` + tests unitarios y de ruta ADR |
| **Commit** | Commits atómicos `6b5170c19` / `cc1ec126d` y `93fae5b5f`, conventional commits |
| **Release** | Incluido en tren `v0.10.0` |

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, nextest)
- codegraph_codegraph_explore (blast radius de callers de `l1_reader`)

**Skills cargadas (SDP):** `api-and-interface-design`, `test-driven-development`, `systematic-debugging`

## Investigation Notes
- Se verificaron todos los callers de `read_namespace_records` y `read_session_records`: `auto_recall` (5 rutas + `recall_candidates`) y `pipeline_worker` (consolidación L2 a escenas) requerían filtrar registros superados; `l1_writer`, `lifecycle` (decay) y auditoría requieren los lectores crudos.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

- **Repro:** Al marcar un registro L1 con `superseded_by: Some("new_key")` (vía contradicción `MEMG-01`), las consultas de `auto_recall` y la consolidación L2 seguían devolviendo el hecho obsoleto porque invocaban `read_namespace_records` / `read_session_records` sin filtrar.
- **Hipótesis:** `l1_reader.rs` carecía de lectores filtrados por vigencia (`is_active_record`) y las rutas de recall en `auto_recall.rs` no descartaban registros con `superseded_by` poblado.
- **1 variable controlada:** Introducir `is_active_record` (`superseded_by` ausente o vacío tras `.trim()`) y los lectores `read_active_*`, migrando las rutas de recall y consolidación L2 a dichos helpers.
- **Test RED:** Test de recall con hecho contradicho devolvía tanto el registro viejo como el nuevo (RED); tras aplicar `read_active_*` e `is_active_record`, solo devuelve el registro vigente (GREEN).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — Impide que hechos revocados o corregidos por contradicción sigan inyectándose en el prompt del agente.
- [x] **PERFORMANCE** — Filtro O(1) en memoria (`is_active_record`) durante el escaneo de registros del namespace/sesión, sin alocaciones adicionales.

## Steps

### Step 1: Implementar `is_active_record` y lectores `read_active_*` en `l1_reader.rs`
- **Archivos:** `vanta-memory/src/core/record/l1_reader.rs`
- **Acción:** Añadir `is_active_record` (tratando `Some("")`/whitespace como activo) y funciones `read_active_namespace_records` / `read_active_session_records`.
- **Verify:** `cargo nextest run --profile audit -p vanta-memory -E "test(l1_reader)" --locked --build-jobs 2`
- **Estado:** ✅ COMPLETED (`6b5170c19`)

### Step 2: Cablear lectores activos en `auto_recall.rs` y consolidación L2 (`pipeline_worker.rs`)
- **Archivos:** `vanta-memory/src/core/hooks/auto_recall.rs`, `vanta-memory/src/pipeline_worker.rs`
- **Acción:** Filtrar registros superados en las 5 rutas de recall y en la entrada de consolidación L2 (`cc1ec126d`).
- **Verify:** `cargo nextest run --profile audit -p vanta-memory -E "test(supersed) or test(recall)" --locked --build-jobs 2`
- **Estado:** ✅ COMPLETED (`cc1ec126d`)

### Step 3: Unificar `recall_candidates` con `is_active_record` y añadir test de ruta ADR
- **Archivos:** `vanta-memory/src/core/hooks/auto_recall.rs`
- **Acción:** Aplicar `is_active_record` en `recall_candidates` (blank pointer = activo) y añadir test de cobertura para la ruta ADR (O7/O8 del review P2-01).
- **Verify:** `cargo nextest run --profile audit -p vanta-memory -E "test(recall)" --locked --build-jobs 2`
- **Estado:** ✅ COMPLETED (`93fae5b5f`)

## Dependencias
- Wave 0 completa.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (`2047a71b`) + re-verificación independiente en auditoría `audit-w2-w3-mem-iql` (#22)
- **Enfoque:** Correcto. Ronda 1 solicitó cubrir también la consolidación L2 (`pipeline_worker.rs:643`), resuelto en `cc1ec126d`, y los opcionales O7/O8 (`recall_candidates` blank-pointer + test ADR) se cerraron en `93fae5b5f`.
- **Cómo se probó:** Suite `vanta-memory` 727/727 PASS + 53/53 tests focalizados en B3 + inspección estática completa en #22.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria.
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado.
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas.
  - [x] Verificar cobertura SDP (v3).
- **Veredicto:** ✅ approve

## Notas
- Commits en `develop`: `6b5170c19` (amend en `cc1ec126d` con fix L2) y `93fae5b5f` (O7-O8).
- Hallazgo hermano en reflexión L2 (`vanta-memory/src/core/reflection/mod.rs:356`) registrado como `FIND-659` en `docs/dev/Backlog.md`.
