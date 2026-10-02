---
title: "HARD-06 — Deuda quick wins (FIND-162, FIND-160, FIND-154, decisión FIND-161)"
kind: task
description: Verificación mecánica del contrato
---

# HARD-06 — Deuda quick wins (FIND-162, FIND-160, FIND-154, decisión FIND-161)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 6 — HARD-06, F0)
- **Fuente:** master-roadmap Task 6 · Backlog L242 (FIND-154), L258 (FIND-160), L259 (FIND-161), L260 (FIND-162) · nota L75 (MEM-55)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴
- **Tipo:** Mixto (scripts/CI/docs)
- **Turns estimados:** 20
- **Creado:** 2026-09-26T19:58
- **last-synced:** 2026-09-27T04:53
- **Estado:** ✅ COMPLETED — 5/5 steps + verify OK + P2-01 ✅ (adversarial, ronda fresca `ses_f1de9b945ffemGMHyUx6K3GDm7`); commit por bloques (LEAD)
- **Incógnitas (uphill):** 0 (FIND-161 resuelto — owner eligió B en Gate P)
- **Pendientes (downhill):** 0 steps (5/5 ✅)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `dev-tools/verify.ps1:86-87` invoca `scripts/validate-docs-coverage.ps1` (fast gate + `/audit quick`) · CI `perf-bench.yml` (push main/develop, paths `src/**`, `vantadb-python/**`, `benchmarks/**`) · consumidores de docs: usuarios + `Check API Docs Version` |
| Callees | Script docs → `src/sdk/*.rs`, `src/config.rs`, `src/error.rs`, `src/cli.rs`, `vantadb-python/src/lib.rs`, `vantadb-mcp/src/handlers/tools.rs`, mirrors de skills · perf gate → `benchmarks/vantadb_local_bench.py`, `benchmarks/python_baseline.json` |
| Implicaciones | FIND-162: el check §3 pasa de inerte a activo → pueden aparecer gaps reales en `EMBEDDED_SDK.md` (se corrigen en el mismo commit, NO se excluye sin motivo). FIND-154: cambia CUÁNDO el gate bloquea (tolerancia documentada) — debe seguir bloqueando regresiones reales. FIND-160/161: docs sin impacto mecánico. Sin API pública, sin migración de datos |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `scripts/validate-docs-coverage.ps1` (227 líneas — §3 en L101-108) · `.github/workflows/perf-bench.yml` (169 líneas — compare L81-150) · `docs/api/VANTA_MEMORY.md` (181 líneas) · `src/error.rs:100-179` (`pub enum Error` L137) · `docs/api/ERROR_HANDLING.md:360-434` (§7 cross-ref) · `docs/api/HTTP_API.md:770-809` (§Error responses) · filas Backlog L242/258/259/260 + L75 · `docs/dev/tasks/MEM-55.md` · `scripts/anti_stutter_map.json` (grep)
- **Archivos referenciados hacia dentro:** script → `docs\api\EMBEDDED_SDK.md`, `docs\user\operations\CONFIGURATION.md`, `docs\api\PYTHON_SDK.md`, `docs\api\MCP.md` + mirrors `skills/`↔`.opencode/skills/` · perf gate → `benchmarks/python_baseline.json` (baseline activo desde 2026-09-25)
- **Archivos que referencian a los editados (referencias entrantes):** `verify.ps1` (gate), CI workflows, `docs/dev/avance/activo/ci-cd.md` (FIND-153), Backlog (filas), `docs/dev/tasks/MEM-55.md` (huérfano)
- **Veredicto impacto:** **medio** — reparar §3 activa un gate hasta hoy trivial; perf gate cambia política de fallo; ambos con verificación mecánica y sin tocar producto.

## Contrato

> "FIND-162: check reparado o retirado con motivo + `rg VantaError scripts/` = 0 (o excepción documentada) Y FIND-160: secciones por módulo O exclusión motivada Y FIND-154: gate ya no falsea (tolerancia/quarantine documentada) Y FIND-161: decisión registrada (ADR o decision-memory) + docs coherentes; filas del Backlog actualizadas/cerradas" (plan master, verbatim).

**Verificación mecánica del contrato:**
1. `pwsh scripts/validate-docs-coverage.ps1` → exit 0 con §3 **no trivial** (variants extraídos > 0) + `rg VantaError scripts/validate-docs-coverage.ps1` → 0 hits.
2. `rg -n "dream|approval_handlers|auto_sync|conversation_hook|memory_generation_log" docs/api/VANTA_MEMORY.md` → secciones/tabla por módulo (o exclusión motivada).
3. `python benchmarks/compare_baseline.py --self-test` → ruido (+108% p50 mismo-commit) NO bloquea; regresión real (+40% métrica estable) SÍ bloquea.
4. Decisión FIND-161 registrada (`rg "FIND-161|RFC 9457" .opencode/task-system/memory/decisions.md` o ADR) + `docs/api/HTTP_API.md`/`ERROR_HANDLING.md` coherentes.
5. Backlog: filas FIND-154/160/161/162 actualizadas/cerradas; FIND-153 verificada-resuelta; MEM-55 con disposición.

## Spec (SDD — decisiones técnicas; FIND-154/FIND-161 con tradeoff real)

> Gate P: FIND-161 dispara UNA ronda de `question` al owner (plan: "FIND-161 ambiguo → opciones + decisión owner si hay tradeoff"). Resto decidido por evidencia.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | FIND-162 §3 | A) reparar el check (`pub enum VantaError {` → `pub enum Error {` en L102 + label L108 + patrón `Error::` en L27) y pagar gaps reales / B) retirar la sección con motivo | A | ✅ decidido-por-evidencia (contrato FIND-162 prefiere reparado; 1 línea + gaps; ref: `scripts/validate-docs-coverage.ps1:101-108`, `src/error.rs:137`) |
| 2 | FIND-154 fix | A) extraer compare a `benchmarks/compare_baseline.py` con bandas por métrica + self-test + quarantine documentada para métricas ruidosas / B) mantener inline y solo subir umbral global (no testeable; reincide) / C) quarantine total (gate advisory — pierde señal) | A | ✅ decidido-por-evidencia (evidencia +108.5% mismo-commit; necesidad de test reproducible; pre-mortem F1 del plan) |
| 3 | Bandas FIND-154 | A) por familia: `query_hybrid.*`/`query_text.*` → borde de ruido (warning) + métricas estables (ingest/throughput) → bloqueo estricto / B) umbral único más alto para todo (pierde detección) | A | ✅ decidido-por-evidencia (evidencia: `query_text.p99` 0.01→0.04ms = µs = ruido, Backlog L242) |
| 4 | FIND-161 envelope | A) mantener envelope actual `{success,error\|data,code(,hint)}` + decisión explícita documentada (consistencia cross-binding API-01; RFC 9457 queda diferido con trigger) / B) adoptar RFC 9457 (`type/title/status/detail/instance`) ahora (surface nueva, HTTP-only, rompe consistencia con bindings, 0 usuarios) | A | ✅ **Gate P ejecutado 2026-09-27: owner eligió B (adoptar RFC 9457)** — decisión en `decisions.md`; implementación ruteada a **FIND-165** (fuera del scope de HARD-06: `src/server/**` prohibido); docs alineados (HTTP_API.md + ERROR_HANDLING §7) |
| 5 | MEM-55 | A) cerrar task file (steps ✅; `conversation_trigger` existe `src/server/state.rs:134` y bootstrap lo pasa `None` L332 = no wireado en prod) + documentar residual / B) catalogar fila Backlog y dejarlo abierto | A | ✅ decidido-por-evidencia (Backlog L75 "verificado conversation_trigger: None"; plan: "catalogar o cerrar") |
| 6 | Secuencia commits | 4 commits atómicos (1 por FIND) + bookkeeping consolidado | ✅ decisión de plan (pre-mortem F3: "commit no atómico → 1 commit por FIND") |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** el fast gate sigue verde tras reparar §3 (los gaps reales se arreglan, no se excluyen sin motivo); el perf gate sigue bloqueando regresiones reales (self-test lo prueba); docs técnicas en inglés (Regla 3); `anti_stutter_map.json` NO se toca a ciegas (registra pares old→new por diseño — excepción documentada); no debilitar el quality bar (CONSTRAINTS.md).
- **Comandos de verificación:** `pwsh scripts/validate-docs-coverage.ps1` · `python benchmarks/compare_baseline.py --self-test` · `pwsh dev-tools/verify_changed.ps1` · `rg VantaError scripts/validate-docs-coverage.ps1`.
- **Deuda pendiente:** ninguna; si FIND-154 no converge (varianza irreducible) → DEFER con evidencia + fila FIND nueva (stop condition del plan).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# HARD-06: Deuda quick wins` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ PENDING (archivo + comando) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | HARD-07 (`docs/dev/tasks/HARD-07.md`) |

`contract` (sub-campos §12.3):
```
contract:
  verificacion: pwsh scripts/validate-docs-coverage.ps1 exit 0 (sin §3 trivial) + python benchmarks/compare_baseline.py --self-test pass
  evidencia:
    - claim: §3 extrae variants (>0) tras el rename
      evidencia: output del script (conteo de items en el bloque Error)
      confianza: alta
    - claim: gate perf ya no falsea con ruido de runner
      evidencia: self-test fixture (ruido → pass; regresión real → fail)
      confianza: alta
  artefactos: [docs/dev/tasks/HARD-06.md, scripts/validate-docs-coverage.ps1, benchmarks/compare_baseline.py, docs/api/VANTA_MEMORY.md, docs/api/HTTP_API.md, docs/api/ERROR_HANDLING.md, docs/dev/Backlog.md]
  invariantes: fast gate verde; perf gate sigue bloqueando regresiones reales
  deuda: FIND-153 cerrada (sin fila); MEM-55 cerrado; anti_stutter json = excepción documentada
  queda_pendiente: Gate P FIND-161 (question owner) antes del commit de la decisión
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (neto negativo — paga FIND-154/160/161/162 + cierra FIND-153/MEM-55; 0 deuda nueva).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable (5 condiciones) + script self-test + validate-docs-coverage exit 0 |
| **Commit** | 4 commits atómicos (1 por FIND) conventional (`fix(scripts):`, `docs(api):`, `fix(ci):`, `docs(api):`) + 1 bookkeeping (`chore:`), diff limpio, sin push |
| **Release** | N/A — sin cambio de producto/publicación; justificar en Notas (FIND-161 se documenta, no se implementa envelope nuevo). Verificación = `verify_changed.ps1` |

## Herramientas necesarias
- Terminal (pwsh/python/cargo), grep/read
- `codegraph_explore` (blast radius)
- campaign MCP (`campaign_update_task_state`, `campaign_verify_cmd`, `campaign_memory_write` para la decisión)
- `gh` CLI (opcional: citar runs del perf gate)

**Skills cargadas (SDP v3, pre-run 2026-09-27):** `campaign-executor` · `progreso` · `ci-cd-and-automation` · `git-workflow-and-versioning` · `documentation-and-adrs` · `api-and-interface-design` · `performance-optimization` · `doubt-driven-development` · **PINNED (policy):** `ci-cd-and-automation`, `git-workflow-and-versioning`, `documentation-and-adrs`, `api-and-interface-design`, `performance-optimization`
`SDP: campaign-executor, ci-cd-and-automation, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, source-driven-development, documentation-and-adrs`

## Investigation Notes
- **FIND-162 (evidencia):** `scripts/validate-docs-coverage.ps1:102` hace `-split '(?<=pub enum VantaError \{)'` → `src/error.rs:137` es `pub enum Error` → `$errorEnumBody` vacío → `$allErrors=@()` → `Check-Methods` reporta "0 items ok" trivial. Hits `VantaError` en el script: L27 (patrón `Test-InDoc`), L102 (split), L108 (label) = 3. `anti_stutter_map.json` 6 hits (L15/61/62/72/102/105) — registra pares old→new por diseño (OD-1/OD-4) → NO tocar (excepción documentada; nota: L15 ref `src/error.rs:122` está stale → :137, y status `proposed` estando completado → posible follow-up, fuera de scope).
- **FIND-160 (evidencia):** `docs/api/VANTA_MEMORY.md` documenta L0-L3/recall/context/offload/ingest/skills/orquestación/gateway pero no: `core/dream/` (existe `vanta-memory/src/core/dream/mod.rs`), `gateway/approval_handlers.rs`, `ingest/auto_sync.rs`, `services/conversation_hook.rs`, `core/memory_generation_log` (path exacto a confirmar — no es `core/memory_generation_log.rs`).
- **FIND-154 (evidencia):** `perf-bench.yml` L88 `THRESHOLD_PCT = 15.0` global, mediana de 3 runs vs `benchmarks/python_baseline.json`; falsos positivos por varianza de runner: `query_hybrid.p50` 5.76→12.01ms (+108.5%), `query_text.p99` µs (Backlog L242). Baseline arbitrado en una máquina. **FIND-153 RESUELTO** 2026-09-25: baseline activado (run `36093538630`, commit `114f55f0`) y verificación real (`##[notice]No regression > 15.0%`, run `36094025761`) → registrar cierre, sin fila pendiente.
- **FIND-161 (evidencia):** `HTTP_API.md:787` declara "Full RFC 9457 ... not implemented — tracked in Backlog (FIND-161)"; envelope real `{success,error|data,code(,hint)}` (L780-785); `ERROR_HANDLING.md:406-420` cross-ref; API-01 fijó "one envelope shape across bindings". `rg problem+json src/server/` = 0 (API-09).
- **MEM-55:** steps ✅ en el task file; `conversation_trigger` existe (`src/server/state.rs:134`, handler `handlers.rs:1391`) pero `bootstrap.rs:332` lo pasa `None` → no wireado en producción (Backlog L75 lo llama huérfano).
- **Gate P FIND-161:** al ejecutar, 1 ronda `question` al owner (A mantener envelope / B adoptar RFC 9457); registrar respuesta en recitation + `campaign_memory_write(file="decisions")`; si el owner elige ADR, aplica Regla 5 (lo articula el owner).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — FIND-161 resuelto (owner eligió B vía Gate P) |
| Pendientes de ejecución (downhill) | 0 — 5/5 steps ✅ (verify OK) |
| % completado | 100% (steps) — pendiente commit por bloques + review P2-01 (LEAD) |

## Fase 1 — Evidencia de Debugging (GATE — FIND-154, tipo `fix(ci)`)

- **Repro:** comparar contra baseline con las métricas del incidente (fixture `benchmark_results.{1,2,3}.json` con `query_hybrid.p50` +108% mismo-commit) → el gate actual (15% global) **falla** (falso positivo); los runs reales: verde `36094025761` (develop) vs rojo `36101773914` (main), mismos commits.
- **Hipótesis:** varianza del runner (mediana de 3 insuficiente para métricas híbridas) + umbral global único + baseline calibrado en otra máquina → no es regresión de código.
- **1 variable controlada:** el umbral/banda de tolerancia por métrica (nada más se cambia en el gate).
- **Test RED:** fixture de ruido reproducido con la lógica actual → exit 1 (RED). Con el fix → exit 0; fixture de regresión real (+40% en métrica estable) → exit 1 antes y después (protege contra debilitar el gate). Ambos casos viven en `--self-test`.
- **Resultado (2026-09-27, HARD-06):** RED confirmado por el incidente real (run `36101773914` falló con el umbral global 15% ante +108.5% de ruido) y reproducido en la fixture del self-test; GREEN con el fix (`--self-test` → 2/2: ruido +108.5% → 0 blocking/2 warnings; regresión +40% estable → 1 blocking; smoke CLI: CASE_A exit 0 / CASE_B exit 1). 1 variable controlada: bandas por familia en `benchmarks/compare_baseline.py` (nada más cambió en el gate — el workflow solo invoca el script).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluado: no aplica a trust boundaries de producto. FIND-161 solo documenta errores (no cambia códigos/status). Sin dependencias nuevas.
- [x] **PERFORMANCE** — aplica parcialmente: FIND-154 ES el gate de performance (CI, no hot path de producto). Se documenta baseline/impacto: sin cambio de runtime; el cambio es política de tolerancia del gate (Regla 9 no aplica al producto).

## Steps

> 4 fixes = 4 commits atómicos (1 por FIND, decisión del plan). Cada step termina con su commit local (sin push — Regla 7).

### Step 1: FIND-162 — reparar §3 de validate-docs-coverage
- **Archivos:** `scripts/validate-docs-coverage.ps1` (+ `docs/api/EMBEDDED_SDK.md` si aparecen gaps)
- **Acción:** L102 `pub enum VantaError \{` → `pub enum Error \{`; L27 patrón `VantaError::$Name` → `Error::$Name`; L108 label `(VantaError)` → `(Error)`; correr el script y pagar los gaps reales que emerjan en `EMBEDDED_SDK.md` (o `Exclude` con motivo uno-a-uno). Excepción documentada: `anti_stutter_map.json` (pares old→new por diseño). Commit `fix(scripts): FIND-162 — check Error variants tras rename`.
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` exit 0 y §3 con `N items ok` (N>0); `rg VantaError scripts/validate-docs-coverage.ps1` → 0 hits.
- **Estado:** ✅ COMPLETED 2026-09-27 — 3 eds aplicadas (L27 patrón `Error::`, L102 split, L108 label). Script exit 0 con §3 **no trivial** ("src/error.rs (Error) — 35 items ok"; 32 variantes reales extraídas + `fn/pub/use` excluidos). `rg VantaError scripts/validate-docs-coverage.ps1` = **0 hits** (V2b: Select-String count = "0"; direct rg exit 1). No emergieron gaps (EMBEDDED_SDK.md ya listaba las 32 variantes, refs L651-670+; test negativo `FooBarBazXYZ` = false → el check detecta faltantes). Verify vía `campaign_verify_cmd` OK (V1b `pwsh scripts/validate-docs-coverage.ps1` pass; V2b pass stdout "0"). Quirk spawn registrado: `pwsh -NoProfile -Command "pwsh ..."` anidado no spawnea (exitCode -1); la forma directa `pwsh scripts/...` sí. Excepción documentada: `scripts/anti_stutter_map.json` (6 hits `VantaError` = pares old→new por diseño, NO se toca).

### Step 2: FIND-160 — módulos operativos en VANTA_MEMORY.md
- **Archivos:** `docs/api/VANTA_MEMORY.md` (+ refs a `vanta-memory/src/core/dream/`, `gateway/approval_handlers.rs`, `ingest/auto_sync.rs`, `services/conversation_hook.rs`, `core/memory_generation_log`)
- **Acción:** agregar sección/tabla por módulo (qué hace + code refs + degradación P4 donde aplique) para los 5 módulos sin cubrir; alternativa admitida: exclusión motivada por módulo (no ambas a medias). Actualizar `last_reviewed`. Commit `docs(api): FIND-160 — módulos operativos vanta-memory`.
- **Verify:** `rg -n "dream|approval_handlers|auto_sync|conversation_hook|memory_generation_log" docs/api/VANTA_MEMORY.md` → 5 términos presentes; `pwsh scripts/validate-docs-coverage.ps1` exit 0.
- **Estado:** ✅ COMPLETED 2026-09-27 — §"Operational modules" agregada (tabla 5 módulos + notas por módulo con code refs + degradación P4/LLM-free; `core::dream` MEM-61, `core::memory_generation_log` MEM-41, `gateway::approval_handlers` MEM-68, `ingest::auto_sync` MEM-45, `services::conversation_hook` MEM-55 con wiring-status residual `None` en `bootstrap.rs:332`) + namespace `dream/<session>/<run_id>` en la tabla + `last_reviewed: 2026-09-27`. Verify: 5/5 términos presentes (conteos 6/2/2/2/2); `validate-docs-coverage` exit 0 (V160c); step verificado vía `campaign_verify_cmd` (V160b AND-check pass). **Nota para el LEAD:** este archivo ya traía cambios sin commitear de HARD-04 (Facade + Exposure triggers) — el bloque FIND-160 incluye solo mi sección nueva (el archivo se commitea completo en el bloque que decida el LEAD).

### Step 3: FIND-154 — perf gate tolerante a varianza
- **Archivos:** `benchmarks/compare_baseline.py` (nuevo), `.github/workflows/perf-bench.yml`, (doc) `docs/dev/avance/activo/ci-cd.md` o comentario en el workflow
- **Acción:** extraer la lógica de compare L81-150 a `benchmarks/compare_baseline.py` con: bandas por familia (ruidosas → warning + quarantine documentada; estables → bloqueo), `--self-test` con fixtures (ruido +108% → pass; regresión real +40% → fail); el workflow invoca el script. Si el fix exige reescribir el gate → DEFER con evidencia (stop condition). Commit `fix(ci): FIND-154 — gate perf con tolerancia de varianza`.
- **Verify:** `python benchmarks/compare_baseline.py --self-test` → 2/2 casos esperados; YAML parse OK (`python -c "import yaml;yaml.safe_load(open('.github/workflows/perf-bench.yml'))"` si pyyaml disponible; si no, `actionlint`/nota); revisar que el step de CI invoca el script.
- **Estado:** ✅ COMPLETED 2026-09-27 — `benchmarks/compare_baseline.py` creado (bandas: estables `STABLE_BLOCK_PCT=15%`; quarantine `query_hybrid`/`query_text` warn >15% + bloqueo solo >300% ceiling catastrófico, documentado en docstring; `--runs/--baseline/--median-out/--update-baseline/--self-test`). Workflow: heredoc inline (68L) reemplazado por invocación del script (comment con bandas + if/else update-baseline). Verify: self-test **2/2 PASS** (ruido +108.5% → 0 blocking + 2 warnings; regresión +40% estable → 1 blocking; `V3a`); YAML OK (`V3b`); smoke CLI con fixtures en temp (CASE_A exit 0 + median escrito; CASE_B exit 1) + check de invocación en workflow (`rg compare_baseline` → L88/90). Doc operativa (avance ci-cd) → Step 5.

### Step 4: FIND-161 — decisión + docs coherentes (Gate P)
- **Archivos:** `docs/api/HTTP_API.md` (§Error responses L778-788), `docs/api/ERROR_HANDLING.md` (§7), `.opencode/task-system/memory/decisions.md` (vía `campaign_memory_write`)
- **Acción:** (1) `question` al owner (1 ronda): A mantener envelope [recomendado] / B adoptar RFC 9457; (2) registrar decisión (`campaign_memory_write(file="decisions", entry="FIND-161 | ... | ref: HTTP_API.md:778")`); (3) actualizar HTTP_API.md para que la nota apunte a la decisión (no "tracked in Backlog") y ERROR_HANDLING §7 alineado. Commit `docs(api): FIND-161 — decisión envelope HTTP (RFC 9457 diferido)`.
- **Verify:** `rg -n "FIND-161" .opencode/task-system/memory/decisions.md docs/api/HTTP_API.md` → decisión registrada y doc coherente; `pwsh scripts/validate-docs-coverage.ps1` exit 0.
- **Estado:** ✅ COMPLETED 2026-09-27 — **Gate P ejecutado: el owner eligió B (adoptar RFC 9457), no A.** Decisión registrada en `decisions.md` (L135) con estado "aprobado, NO implementado → implementación ruteada a **FIND-165**". Docs: `HTTP_API.md:787-793` nota reescrita (adopted 2026-09-27 + dirección problem+json + wire contract vigente hasta que aterrice); `ERROR_HANDLING.md:422-426` cross-ref alineado. RFC 9457 verificado contra fuente oficial (`rfc-editor.org/rfc/rfc9457.html`, fetch OK — Gate citas). Verify: V161c pass (decisión + docs) + V161b pass (validate-docs exit 0). **Nota para el LEAD:** el commit de este bloque será `docs(api): FIND-161 — decisión: adoptar RFC 9457 (implementación ruteada a FIND-165)` (no "RFC 9457 diferido" — el owner eligió B). Implementar el envelope NO entra en el scope de HARD-06 (archivos prohibidos src/server/**) → FIND-165.

### Step 5: Bookkeeping (filas Backlog + FIND-153 + MEM-55) + verify final
- **Archivos:** `docs/dev/Backlog.md`, `docs/dev/tasks/MEM-55.md` (→ `docs/dev/tasks/complete/` si se decide cerrar), `docs/dev/avance/activo/ci-cd.md`
- **Acción:** actualizar/cerrar filas FIND-154/160/161/162; verificar y registrar FIND-153 resuelta (baseline no vacío + run verde `36094025761`); disposición MEM-55 (cerrar con nota de residual `conversation_trigger=None` en `bootstrap.rs:332` o catalogar); correr verify final. Commit `chore(backlog): cierre FIND-153/160/161/162 + MEM-55`.
- **Verify:** `rg -n "FIND-15[34]|FIND-16[012]|MEM-55" docs/dev/Backlog.md` → estados actualizados; `pwsh dev-tools/verify_changed.ps1` verde; `pwsh scripts/validate-docs-coverage.ps1` exit 0.
- **Estado:** ✅ COMPLETED 2026-09-27 — filas FIND-154/160/161/162 → ✅ Cerrado (HARD-06) con detalle; **FIND-165 creada** (implementación RFC 9457, ruteada desde FIND-161); **FIND-153**: sin fila en Backlog (nunca catalogada como pendiente) — cierre en `ci-cd.md` (§FIND-153 ✅) + evidencia live `gh run view 36094025761` → `conclusion: success` (develop, workflow_dispatch); **MEM-55**: nota L75 con disposición ✅ + `MEM-55.md` → `docs/dev/tasks/complete/MEM-55.md` con bloque de cierre (residual `conversation_trigger: None` en `bootstrap.rs:332`); `ci-cd.md` §FIND-154 con línea de Resolución. Verify: `verify_changed.ps1` **ALL 4 PASS** (fmt/check/clippy/docs-coverage); Backlog rg OK; OCR advisory sin findings Critical/High/Medium en los 3 archivos del spec propios del task. Commit pendiente: `chore(backlog): cierre FIND-153/160/161/162 + FIND-165 + MEM-55` (LEAD).

## Dependencias
- Sin bloqueos. FIND-153 ya resuelta (2026-09-25, commit `114f55f0`). Coordina con HARD-02 (mismo verify) sin solape de archivos.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` (contexto fresco) o `vanta-audit`. *(PENDIENTE al ejecutar — placeholder P2-01.)*
- **Enfoque:** ¿la reparación del §3 no introduce falsos gaps ni exclusiones oportunistas? ¿el perf gate sigue detectando regresiones reales tras la tolerancia? ¿la decisión FIND-161 quedó registrada y coherente?
- **Cómo se probó:** re-ejecución del revisor (script + self-test + conteos), no auto-reporte.
- **Checklist anti-hábitos tóxicos** (contrato de comportamiento — el revisor verifica que el implementador NO haya incurrido en ninguno antes de aprobar; fuente §12 de `docs/Investigaciones/2026-08-10-agent-engineering/agent-02-task-execution.md`):
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
- **Revisor:** vanta-review (sesión `ses_f1de9b945ffemGMHyUx6K3GDm7`, contexto fresco) — **tier adversarial** (diff toca `docs/api/**`)
- **Enfoque:** no-trivialidad de §3 (FIND-162), bandas FIND-154, coherencia RFC 9457 (FIND-161), attribución de scope
- **Cómo se probó:** re-ejecución directa por el revisor (script exit 0 + `rg` counts + self-test 2/2 + smoke propio con fixtures + `gh run view` + rfc-editor.org), no auto-reporte.
- **Veredicto:** ✅ APPROVE — 0 Critical / 0 Required; 2 optional (trigger de warnings sostenidos en cuarentena; attribution del plan file compartido) + 1 nit (display "35 items" vs 32 reales en el script).

## Notas
- **Rollback-friendly:** cada FIND es un commit independiente y revertible (~100 líneas c/u).
- **Riesgo FIND-154:** si la varianza es irreducible → quarantine + fila FIND (stop condition del plan, trigger: 2 falsos positivos más).
- **NOTICED BUT NOT TOUCHING:** `scripts/anti_stutter_map.json` L15 (`ref: src/error.rs:122` stale / status `proposed` pese a completado) — el archivo registra pares old→new por diseño; ajustarlo pertenece al tooling anti-stutter (AST-*). Follow-up opcional vía `FIND-*` si el owner lo pide.
- **No debilitamiento del quality bar:** FIND-162 REPARA un gate (no lo retira); FIND-154 conserva bloqueo en métricas estables (self-test lo prueba).
- Memoria del plan: "FIND-153 se evalúa junto a FIND-154" → Step 5 lo cubre (verificación + registro, sin re-fix).
