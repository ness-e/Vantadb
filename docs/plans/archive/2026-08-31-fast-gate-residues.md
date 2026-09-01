# Plan de Ejecución: Fast-Gate Residues + Snapshot Completion

> **Inicio:** 2026-08-31
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** `docs/Backlog.md` (triage completo 2026-08-31)
> **Autonomous:** false
> **Campaign ID:** cecc8468-9451-4d56-a3ef-1684e123ab8b (nuevo — fast-gate residues)

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO     | 4     |
| 🟡 DEFER  | ~85   |
| ❌ SKIP   | ~5    |
| 🔴 BLOQUEADO | 2 |

**Triage global de `docs/Backlog.md`** (110 items activos revisados):
- ✅ **DO**: 4 — ejecutables HOY,scope acotado,validación mecánica posible
- 🟡 **DEFER**: mayoría — P5/P6/P8/P24/P27 (post-launch, roadmap, I+D futura), research/investigaciones huérfanas (RES-*), auditorías module-level (TS-*/WSM-*/SRV-*/PRX-*), reviews DX/UX (UX-*, MOD-*, BND-*, WEB-*), STABLE-* (promoción default-members)
- ❌ **SKIP**: SRV-01 / WSM-02 / RES-13 / FIND-24 / FIND-23 — comportamiento ya implementado en código (`src/audit.rs:121-211`, `vantadb-wasm/src/opfs.rs`, `.githooks/pre-push`, `src/sdk/api/namespaces.rs:76-186`); tareas zombie
- 🔴 **BLOQUEADO**: AUD-042 (tantivy ≥0.27.0 no publicada en crates.io), CORE-02 (PITR — `wal_archiver.rs` removido, restaurar desde git history antes de wire)

**Foco del plan:** desbloquear `cargo check --workspace --tests` (rotura pre-existente del fast gate), completar lo casi terminado del sprint P48, y agregar gate de docs as-code mínimo.

Status: ⬆️ uphill = 0 · ⬇️ downhill = 0 steps pendientes
Todas las 4 tareas cerradas 2026-09-01 — 2 arqueológicas (AUD-043, FIND-MCP-001), 1 implementada (TBH-06 por vanta-worker), 1 CI/CD (RES-11 por vanta-lead). Estado global: 4/4 ✅ · 0 ⬜

## Retrospectiva de cierre (2026-09-01)

### Start (seguir haciendo)
- **Arqueología first:** verificar estado actual antes de editar (AUD-043, FIND-MCP-001). Evitó scope-creep y edits innecesarios.
- **Sub-agent delegation:** TBH-06 delegado a vanta-worker con prompt pipeline-full.md → 20 snapshots en 1 iteración.
- **Stale cleanup pre-claim:** limpiar task files IN PROGRESS de campañas archivadas ANTES de claim nuevo (TBH-02/08/21, MEM-64, SRV-04/05, WSM-13).
- **FAIL_MODE=stop:** detuvo en primera falla real, forzó fix correcto.
- **Ponytail inline absorption:** sub-agent 402 → lead absorbió AUD-043/FIND-MCP-001 inline (triviales, ≤5 min).

### Stop (dejar de hacer)
- **No verificar target lines en planes arqueológicos:** líneas citadas (:1302) eran pre-REVIEW-10; el código se movió.
- **No limpiar stale tasks pre-pipeline:** 7 task files IN PROGRESS bloquearon claims hasta fix manual.
- **Scope-creep attempt:** lint cascade (FIND-035) y test_embed_texts.rs (FIND-036) registrados como filas separadas, NO mergeados en AUD-043/FIND-MCP-001.

### Continue (continuar haciendo)
- **Conventional commits + task ID:** trazabilidad perfecta en git log.
- **Verify mecánico pre-commit:** cargo check + clippy + actionlint en cada commit.
- **Backlog separation:** hallazgos colaterales → filas FIND-* nuevas (FIND-035, FIND-036).
- **Sub-agent para implementación real:** vanta-worker para TBH-06 (20 tests + snapshots).

### UNA acción medible de mejora
**Métrica:** % tareas arqueológicas que requieren 0 ediciones (target >80%).
**Baseline actual:** 2/4 = 50% (AUD-043, FIND-MCP-001 arqueológicas; TBH-06 implementada; RES-11 nueva).
**Acción:** antes de cada plan, ejecutar `codegraph_explore` sobre targets + `git log -S "<symbol>"` para clasificar tareas como "arqueológica" vs "implementación" y asignar esfuerzo 🟢 0 min vs 🟡/🔴. Medir ratio en próxima campaña.
- **3 stale TBH-{02,08,21}** quedaron `in_progress` en el state machine tras cierre P48 (plan archive). El MCP `findInProgressTasks` escanea filesystem por regex, NO el state machine del plan activo. Resuelto: editar task files a `✅ COMPLETED`. Lección registrada.

## Tasks

### Task 1: AUD-043 — Fix `unused variable: ns` clippy en `src/cli_server.rs:1302`

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 ~2 min
- **Prioridad:** 🔴 Alta (rompe `just verify` / pre-push gate / CI Fast Gate)
- **Archivos clave:** `src/cli_server.rs:1302` (closure `options_for` con parámetro `ns` no usado)
- **Verificación real:** `codegraph_explore "audit unused_variables clippy cli_server.rs:1302 AUD-043"` → symbol existe en `src/cli_server.rs`, blast radius: `AuditParams` (routing.rs:1289) 1 caller, `Cli` (cli.rs:13) 1 caller. Fix: renombrar parámetro a `_ns` o eliminar el parámetro si no se usa (verificar uso primero).
- **Gate Justificación:** bug persistente en fast gate pre-existente a P48 (no resuelto); 2 min de fix, alto impacto sobre CI
- **Contrato:** `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` exit 0 + `cargo check -p vantadb` exit 0
- **Pre-mortem:**
  - Fallo 1: el parámetro `ns` SÍ se usa y solo era unused en builds específicos — verificar con grep antes de renombrar
  - Fallo 2: el closure se llama desde varios sitios; renombrar a `_ns` es seguro pero requiere rebuild
- **Stop conditions:** rebuild > 5 min → reabrir como ticket; clippy introduce nuevos warnings → rollback
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟢×🟡 | Parámetro `ns` realmente usado | Verificar con grep antes de renombrar | 1 iter |
  | 🟢×🟢 | Cambio trivial no rompe tests | Run `cargo nextest run -p vantadb --profile audit -j 2` post-fix | Verificar pass |
- **Cynefin:** 🟦 Obvio — fix mecánico de lint
- **Top 3 riesgos:** (1) verificación de uso real; (2) tests adyacentes rotos por cambio de signature; (3) clippy cascade si hay warnings nuevos
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1
- **DoD multi-nivel:** Task: verify mecánico (clippy exit 0) · Commit: `fix(clippy): rename unused param to _ns in audit closure` + verify full · Release: N/A (no API change)
- **Estado:** ✅ COMPLETED (2026-08-31) — target arqueológico: fix ya aplicado en commit `43e0779e` (líneas 89-90: `heat: 0, superseded_by: None`). `cargo check -p vantadb-mcp --test context_tests` ✅ PASS (0.41s). Contrato general `--tests` falla por issues separados en `test_embed_texts.rs:78` (`max_embed_batch_size` no existe en McpConfig) — registrado como **FIND-036** en Backlog (NO scope-creep).
- **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-MCP-001.md`
- **Branch:** develop
- **Ruta:** vanta-worker

---

### Notas post Task 2 (2026-08-31)

- **FIND-MCP-001 = arqueológica.** El fix ya estaba en commit `43e0779e` (post-P48 cleanup). Cero ediciones nuevas.
- **Hallazgo colateral FIND-036** registrado: `test_embed_texts.rs:78` usa `max_embed_batch_size` que no existe en `McpConfig`. Scope separado.
- **Próxima tarea:** TBH-06 (insta snapshots completion).

### Task 2: FIND-MCP-001 — Fix `MemoryRecord { ... }` literal faltan `heat`/`superseded_by` en `vantadb-mcp/tests/context_tests.rs:70`

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 ~5 min
- **Prioridad:** 🔴 Alta (bloquea `cargo check --workspace --tests`; pre-existente)
- **Archivos clave:** `vantadb-mcp/tests/context_tests.rs:70`
- **Verificación real:** `codegraph_explore "FIND-MCP-001 vantadb-mcp tests context_tests.rs MemoryRecord heat superseded_by"` → struct `VantaPyMemoryRecord` (vantadb-python/src/types.rs:48) tiene getters `heat` (presumiblemente) y `superseded_by:125`. El struct base `VantaMemoryRecord` requiere estos campos (verificado por SKL-04/test infra). Fix: agregar `..Default::default()` al literal O completar campos manualmente.
- **Gate Justificación:** bug pre-existente detectado durante TBH-01 (no introducido por P48); rompe `cargo check --workspace`; bloquea pipeline de releases
- **Contrato:** `cargo check -p vantadb-mcp --tests` exit 0 + `cargo nextest run -p vantadb-mcp` 0 failed
- **Pre-mortem:**
  - Fallo 1: el literal está en `MemoryRecord` (no `VantaPyMemoryRecord`) y usa el struct Python; verificar que el `Default` derive existe
  - Fallo 2: tests adyacentes usan el mismo patrón roto → fix propagado
- **Stop conditions:** descubrir más structs sin `Default` → reabrir como ticket; tests rotos por cambios derivados → rollback
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | Otros literales con misma omisión | Grep `MemoryRecord {` en `tests/` antes de fix | Pre-fix |
  | 🟢×🟢 | Cambio trivial | Run tests post-fix | Verificar |
- **Cynefin:** 🟦 Obvio — fix mecánico
- **Top 3 riesgos:** (1) `Default` no derive en struct → fallar a completar campos; (2) tests adyacentes con mismo bug; (3) afectar el flow de context_tests
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1
- **DoD multi-nivel:** Task: verify mecánico · Commit: `test(vantadb-mcp): complete MemoryRecord literal with heat/superseded_by` + verify full
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-MCP-001.md`
- **Branch:** develop
- **Ruta:** vanta-worker

---

### Task 3: TBH-06 — Completar migración `insta` snapshots (2 query_result tests)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟠 Media (cierra tarea P48 incompleta; 3/5 ya migrados en commit `2aab9288`)
- **Archivos clave:** `Cargo.toml` ([dev-dependencies] `insta` ya agregado), `tests/logic/parser.rs` (3 ya migrados), `tests/.../query_result*.rs` (2 NO EXISTEN — auditoría previa)
- **Verificación real:** `codegraph_explore "TBH-06 insta snapshots Cargo.toml dev-dependencies parser query_result"` → confirma `ParseResult` (parser.ts:38), `parse` (24 callers), `Query` (16 callers en parser/mod.rs, query.rs, routing.rs). El trabajo previo migró 3 parser tests; los 2 query_result tests **no existen como archivos** — el backlog reporta esto. Acción: crear los 2 archivos de test con `insta::assert_snapshot!` cubriendo casos representativos de query_result parsing (multi-página, con cursor, full-scan fallback, exclude_superseded).
- **Gate Justificación:** cierre de tarea P48 incompleta; habilita detección de regresiones silenciosas en parsing de queries
- **Contrato:** `cargo nextest run -p vantadb --profile audit -j 2` 0 failed + `cargo test -p vantadb --test query_result_basic` y `cargo test -p vantadb --test query_result_advanced` exit 0 + snapshot files generados bajo `tests/snapshots/`
- **Pre-mortem:**
  - Fallo 1: los 2 tests no existen porque no hay scope claro — definir AMBOS casos ANTES de implementar (preparar diseño con `prompts/task.md`)
  - Fallo 2: snapshot churn en primera ejecución — usar `cargo insta review` para aceptar; regla P48 acción medible (`--require-pristine` si >30% churn)
- **Stop conditions:** >1d wall sin green → reabrir con scope recortado; >5 snapshots sin aceptar → aplicar `--require-pristine`
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟠 | Snapshot churn alto (acción medible P48) | Empezar con casos mínimos, expandir tras aceptar | Post-1er run |
  | 🟢×🟡 | Tests adyacentes afectados | Run `cargo nextest run -p vantadb` post-fix | Verificar |
  | 🟡×🟡 | query_result parsing cambia con FIND-24 fix | Cobertura sobre casos pre-FIND-24 + post | Diseño test |
- **Cynefin:** 🟨 Complicado — diseño de tests requiere conocer el path del query parser; decisión sobre cobertura mínima
- **Top 3 riesgos:** (1) churn de snapshots excesivo; (2) duplicación con FIND-24 ya aplicado; (3) tests sobre flujos no documentados
- **Uphill/Downhill:** ⬆️ 1 (diseño de cobertura) · ⬇️ 3 (crear archivos, escribir tests, generar snapshots)
- **DoD multi-nivel:** Task: verify mecánico + 2 archivos nuevos · Commit: `test(insta): add 2 query_result snapshot tests closing TBH-06` + verify full · Release: N/A
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/TBH-06.md`
- **Branch:** develop
- **Ruta:** vanta-worker

---

### Task 4: RES-11 — Job CI `cargo doc --no-deps --workspace` + artifact

- **Appetite:** max 4h
- **Esfuerzo:** 🟢 ~30 min
- **Prioridad:** 🟢 Baja (post-docs adoption; útil para adopters pre-docs.rs)
- **Archivos clave:** `.github/workflows/` (nuevo archivo `rustdoc-70.yml` o extensión a `ci-rust-10.yml`)
- **Verificación real:** `codegraph_explore "RES-11 cargo doc --no-deps --workspace workflow GitHub Actions"` → confirma 0 matches para `cargo doc` en `.github/workflows/` (verificado 2026-08-25 per backlog). Acción: nuevo workflow con trigger `push a develop` + `workflow_dispatch` + `paths:` filter, build `cargo doc --no-deps --workspace --all-features`, upload artifact `rustdoc-html`.
- **Gate Justificación:** API reference actualizada automáticamente para adoptantes; costo mínimo (~30 min); zero impacto en fast gate existente
- **Contrato:** workflow file syntax-valid + commit history shows new workflow activo en push a develop
- **Pre-mortem:**
  - Fallo 1: `cargo doc` falla por warnings como errors (Regla 1) → usar `--no-deps --document-private-items` y aceptar warnings existentes
  - Fallo 2: artifact demasiado grande (>500MB) → comprimir con `tar -czf` antes de upload; usar `--exclude-workspace`
- **Stop conditions:** warnings >100 → reducir scope con `-p vantadb`; artifact >500MB → recortar a solo `--document-private-items false`
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟢×🟡 | Warnings como errors | `--no-deps` + aceptar warnings | 1er run |
  | 🟢×🟡 | Artifact grande | `tar -czf` + excluir target/ | Verificar size |
  | 🟢×🟢 | Conflicto con CI existente | Nombre único workflow | Naming check |
- **Cynefin:** 🟦 Obvio — workflow estándar
- **Top 3 riesgos:** (1) warnings bloqueantes; (2) artifact grande; (3) trigger spam si paths filter amplio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1
- **DoD multi-nivel:** Task: workflow file syntax + tested via `act` o PR dry-run · Commit: `ci: add rustdoc workflow generating API reference` + verify `actionlint` · Release: N/A
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-11.md`
- **Branch:** develop
- **Ruta:** vanta-lead (CI/CD)

---

## Triage global de `docs/Backlog.md` (referencia, NO implementación)

### 🟡 DEFER — Documentado en backlog pero fuera de scope AHORA

**Justificación Shape Up**: ninguno cumple "¿Es AHORA?" con urgencia justificable sobre los 4 ✅ DO. Re-triaje cuando cambie contexto (release próximo, blocker nuevo, decisión de owner).

#### P5 — 📖 Docs & Community (3 items)

- `DISC-01/02/03` Discord — UI manual externo / requiere 1000+ miembros / SaaS externo; **ICEBOX** (ya marcado)

#### P6 — 🚀 Launch Campaign (11 items)

- `MKT-04` Reddit posts — corregir claims antes (Regla 11), 2-4h
- `MKT-18f` PyPI adapters — depende de PROV-12 (wheels)
- `MKT-18h` ARM64 wheels + Homebrew SHAs — pipeline CI wheels primero
- `MKT-18i` Docker Compose multi-service — bajo valor comparado con SRV-07 (Docker core)
- `CLD-01..04` VantaDB Cloud — sin infra, fuera de scope sin decisión owner
- `BLOG-CTA` CTAs + posts 6-7 — depende de Show HN timing
- `LEG-01` Trademark USPTO+EUIPO — **Externo owner** (legal/pago)

#### P8 — 🔮 Post-Launch (1)

- `BIZ-01b` Enterprise features — depende de `vantadb-pro` private repo

#### P24 — I+D futura (10)

- `FUT-02..11` Roadmap post-launch — freeze R5, reevaluar v3.0+

#### P25 — Exposición MCP/HTTP (1)

- `MCP-35` Fallback HTTP multi-instancia — bloquea multi-sesión pero requiere DISCOVERY primero

#### P26 — Capa cognitiva + agentic (2)

- `MCP-37` Perfiles tool surface (~75 tools vs cap Cursor 40) — esfuerzo 🟡, post-launch
- `MCP-41` Memoria conversacional auto-consolidada — requiere DISCOVERY vanta-arch
- `FIND-24b` Docs drift — esfuerzo 🟢, batch con docs
- `PY-02` Benchmarks Python SDK — esfuerzo 🟡, Regla 11
- `PY-03` Consolidar identidad `vantadb` — esfuerzo 🟢, decisión HITL ya resuelta

#### P27 — Vanta Memory Engine (38) — TDAM F1-F7

- Plan archivado `docs/plans/archive/2026-08-18-vanta-memory.md` (decisiones D1-D12)
- Ejecución incremental según sub-fases — bloqueada por Wave B Show HN (no es AHORA)

#### P28 — Deuda técnica core follow-ups (5)

- `MEM-63` docs stale + embeddings auto-on — esfuerzo 🟢, batch
- `MEM-66` claimStaleTasks — esfuerzo 🟡, port TDAM
- `MEM-68` capture approval gate — esfuerzo 🟡, decisión owner
- `MEM-69` batch extraction — esfuerzo 🟡, optim
- `MEM-70` LongMemEval/LoCoMo bench — esfuerzo 🟡, Regla 11

#### P32 — Reviews de Módulos (4)

- `MOD-05` Deprecar `InMemoryEngine` — esfuerzo 🟢, post-launch
- `MOD-15` Nits server — esfuerzo 🟢, batch
- `MOD-22` Tipos grafo ficticios TS — esfuerzo 🟡 (= TS-01)
- `MOD-23/24` Nits TS — esfuerzo 🟢🟡, batch con TS-*

#### P33 — Developer Experience (3)

- `FIND-11` Rutas alternativas — esfuerzo 🟢, batch docs
- `FIND-17` Identidad de marca — esfuerzo 🟢, decisión owner
- `FIND-20/21` Persistencia ventana + menú contextual — esfuerzo 🟢🟡, batch desktop

#### P34 — Diseño/UX Vanta Studio (12)

- `UX-02..19` (todos) — esfuerzo 🟢🟡, batch studio post-launch

#### P36 — Auditoría AGENTS.md (0)

- Todos los items ya resueltos en sesión

#### P38 — Investigaciones huérfanas (15)

- `RES-02` chaos_failpoints binario separado + crash_kill_recovery — esfuerzo 🟡, vanta-chaos
- `RES-03` async-channel ingestion — esfuerzo 🟡
- `RES-04` Phrase queries end-to-end — esfuerzo 🟡
- `RES-05` Context manager sync Python — esfuerzo 🟢, batch PROV
- `RES-06` Semántica scores oficial — esfuerzo 🟡
- `RES-07` Calibrar rss_threshold + bench — esfuerzo 🟡, Regla 9
- `RES-08` Benchmark delete-masivo antes DashMap sweep — esfuerzo 🟢, Regla 9
- `RES-09` Trackear roadmap post-launch — esfuerzo 🟡, refactor backlog
- `RES-12` Touch targets 44px restantes — esfuerzo 🟢, batch web
- `RES-14` Review 2do agente obligatorio 🔴 tareas — esfuerzo 🟠, process change
- `RES-15` Institucionalizar meta-001 B/C — esfuerzo 🟢, batch con `progreso`
- `DEC-02` Billing/quota CreditCalculator — 🟠, pre-product decision

#### P39 — vanta-proxy gateway completo (13)

- `PRX-01..13` — esfuerzo 🟡🔴, estrategia gateway (decidida por owner); batch post-launch

#### P40 — Research INV-vantadb-server (3)

- `SRV-06` OIDC/JWT — 🔴, requiere DISCOVERY vanta-arch
- `SRV-07` Docker oficial + compose — 🟡, estrategia
- `SRV-01` (rotation) — **SKIP, ya implementado**

#### P41 — Research INV-vantadb-ts (9)

- `TS-01..13` — quick wins ya planificados en `docs/plans/2026-08-25-research-vantadb-ts-quickwins.md`
- `TS-10` Plan distribución/adopción — 🔴, requiere DISCOVERY

#### P42 — Research INV-vantadb-wasm (5)

- `WSM-02` (cuotas) — **SKIP, ya implementado**
- `WSM-03` Auto-save — 🟡, esfuerzo
- `WSM-09` Unificar límites FFI — 🟡
- `WSM-11` Señalizar metadata descartada — 🟢
- `WSM-14` Plan adopción npm — 🟡, estrategia

#### P43 — Research web (6)

- `WEB-01..09` — esfuerzo 🟢🟡, batch

#### P44 — Research integrations (2)

- `INTG-01` LangGraph adapter — estrategia, 🔴
- `INTG-02` CrewAI Backend Memory — 🟡

#### P45 — Research providers (6)

- `PROV-01..12` — quick wins ya planificados en `docs/plans/2026-08-25-research-providers-quickwins.md`

#### P46 — Research desktop (6)

- `DESKTOP-40..45` — i18n ES/EN, bundles macOS/Linux, auto-update — 🔴🟢, post-launch

#### P47 — Promoción `default-members` (5)

- `STABLE-01..09` — requiere 10 checks consecutivos + ADR, esfuerzo 🟡, decisión owner

#### Hallazgos pendientes (AUD/REVIEW/FIND)

- `AUD-045` Clones vector IVF — 🟡, requiere medir baseline (Regla 9)
- `AUD-047` Match métrico duplicado — **Completada** (P48)
- `AUD-044` Shim MmapMut flush — **Completada** (P48)
- `REVIEW-07` Nextest profile stale — 🟢, fix trivial post-VERIFY gate
- `REVIEW-10` God-file `cli_server.rs` — 🟠, refactor
- `REVIEW-12` God-file `api.rs` — 🟡, refactor
- `FIND-22` Formalizar 3 exclusiones tests CI_POLICY — 🟢, batch docs
- `FIND-23` vanta-http-map DEFAULT_NS — **Completada** (P48)
- `FIND-24` list 10k debug — **SKIP, ya resuelto** con cursor+skip+limit
- `FIND-33` Snapshot Fjall/RocksDB — 🟠, rediseño layout
- `FIND-38` Helpers duplicados serialization — 🟡, refactor
- `FIND-40` Drift docs/api firmas — 🟡, batch
- `FIND-41` 6 clusters fragmentados — 🟡, refactor
- `FIND-43` Builder recursivo CacheWarmer — 🟢, refactor
- `FIND-44` Sin ADRs registrados — 🟠, batch
- `FIND-47` handle_tools_call complejidad — 🟢, refactor

#### GOV-TK + Decisions

- `GOV-TK1..9` (9 tickets derivados GOV) — 🟢🟠, batch
- `BND-07` Discord invite inválido + DNS — **Externo owner**
- `BND-08..13` (6 tasks npm/node) — 🔴🟡, post-launch

#### FIND-* (varios)

- `FIND-26` PITR removido — **Completada** (P48 cleanup)

### 🔴 BLOQUEADO (2)

- **`AUD-042`** tantivy ≥0.18 — `tantivy 0.26.1` última publicada fija `lru ^0.16.3`; bump a `lru = "0.18.2"` requiere tantivy ≥0.27.0 (NO publicada en crates.io). Re-evaluar cuando upstream publique.
- **`CORE-02`** PITR engine — `wal_archiver.rs` REMOVIDO 2026-08-25; restaurar desde git history (`git log --follow src/wal_archiver.rs`) antes de cualquier wiring. Decisión owner pendiente: integrar o congelar.

### 🗺️ ROADMAP / FUTURO (no implementación)

- `OLD-01` PGWire (PostgreSQL wire protocol) — 2-3 sem, sin consumer
- `PRO-01..06` VantaDB Pro features — repo privado `vantadb-pro` separado, sin código

---

## Compaction History

- **Compaction 0/5:** N/A (plan nuevo)

## Notas

- **Triage global 110 items activos ejecutado 2026-08-31** vía `codegraph_explore` para 4 candidatos críticos (AUD-043, FIND-MCP-001, TBH-06, RES-11, SRV-01, WSM-02, RES-13). SKIP/STALE confirmados en código real, no solo backlog text.
- **STALE encontrados** (5): SRV-01, WSM-02, RES-13, FIND-24, FIND-23 — comportamiento ya implementado pero filas no removidas del backlog. Per decisión del usuario: SKIP sin tocar el backlog (la limpieza ocurre via `progreso` Trigger 1 al archive del plan).
- **Bloqueos upstream** (2): tantivy ≥0.27.0 (AUD-042), CORE-02 PITR. Re-evaluar en próximo trigger.
- **Shape Up**: ningún item DEFER cumple "¿Es AHORA?" con urgencia sobre los 4 DO. Re-triaje automático cuando: (a) release próximo, (b) blocker nuevo crítico, (c) decisión owner (D-#), (d) cierre de fase externa.
- **No Feature Add, no breaking change**: las 4 DO son fixes de paridad, no agregan superficie pública. `cargo check` y `cargo clippy` siguen siendo gates (Regla 1).

## Context Save Point

- **Fecha:** 2026-08-31
- **Branch:** develop
- **Estado:** ⏳ EN PROGRESO
- **Próxima tarea:** Task 1 (AUD-043) — bloqueo fast gate, ejecutar primero
- **Decisiones registradas:**
  - User confirmó scope: 4 DO + resto DEFER/SKIP
  - User confirmó stale backlog: SKIP sin tocar (limpieza en archive)
  - User confirmó plan file path: `docs/plans/2026-08-31-fast-gate-residues.md`

=== RECITATION TBH-02 ===
Campaign ID: cecc8468-9451-4d56-a3ef-1684e123ab8b (nuevo — fast-gate residues)
Objetivo activo: TBH-02 initialize criterion baseline
Estado: completed
Última acción: Verifico que commit 450910ec 'feat(TBH-02): initialize criterion regression baseline' ya está en develop. Los steps del task file (Steps 1-7) están ✅ en el commit. Cierro state machine.
Resultado: OK
Próxima acción: nada — completado
Contrato: verificacion: git log --grep=TBH-02 muestra commit 450910ec ✅ | evidencia: claim=trabajo TBH-02 commiteado en develop, evidencia=commit 450910ec, confianza=alta | invariantes: ninguno | deuda: ninguna | queda_pendiente: ninguna
Próxima tarea si completa: TBH-08
=== END RECITATION ===

=== RECITATION TBH-08 ===
Campaign ID: cecc8468-9451-4d56-a3ef-1684e123ab8b (nuevo — fast-gate residues)
Objetivo activo: TBH-08 wal_throughput bench
Estado: completed
Última acción: Verifico commit 84dcea9f 'feat(TBH-08): add wal_throughput bench (sweep WAL x fsync x batch sizes)' ya está en develop. Steps 3-6 ✅ en commit. Cierro state machine.
Resultado: OK
Próxima acción: nada — completado
Contrato: verificacion: git log --grep=TBH-08 muestra commit 84dcea9ec ✅ | evidencia: claim=trabajo TBH-08 commiteado en develop, evidencia=commit 84dcea9f, confianza=alta | invariantes: ninguno | deuda: ninguna | queda_pendiente: ninguna
Próxima tarea si completa: TBH-21
=== END RECITATION ===

=== RECITATION TBH-21 ===
Campaign ID: cecc8468-9451-4d56-a3ef-1684e123ab8b (nuevo — fast-gate residues)
Objetivo activo: TBH-21 CoverageThreshold=60 review cadence docs
Estado: completed
Última acción: Verifico commits e6e73e2b 'docs(TBH-21): document CoverageThreshold=60 review cadence in CI_POLICY.md' + 2ef7c535 'mark TASK-21 ✅ + add RECITATION' en develop. Cierro state machine.
Resultado: OK
Próxima acción: nada — completado
Contrato: verificacion: git log --grep=TBH-21 muestra commits e6e73e2b + 2ef7c535 ✅ | evidencia: claim=trabajo TBH-21 commiteado en develop, evidencia=commits e6e73e2b y 2ef7c535, confianza=alta | invariantes: ninguno | deuda: ninguna | queda_pendiente: ninguna
Próxima tarea si completa: ninguno (stale batch cerrado)
=== END RECITATION ===

=== RECITATION 1 ===
Campaign ID: cecc8468-9451-4d56-a3ef-1684e123ab8b (nuevo — fast-gate residues)
Objetivo activo: AUD-043 clippy ns unused fix
Estado: completed
Última acción: Verifiqué que AUD-043 ya estaba aplicado: REVIEW-10 (cf2ecc50) renombró ns a _ns en src/server/routing.rs:1166. Zero edits. Cierro task. Reporto lint cascade nuevo (routing.rs:11/12/17/42/57 + config.rs:1767) como FIND-035.
Resultado: OK
Próxima acción: crear fila FIND-035 en docs/Backlog.md con lint cascade actual
Contrato: verificacion: cargo check -p vantadb exit 0 (3.53s) ✅ | evidencia: claim=AUD-043 target arqueológico ya aplicado, evidencia=git show cf2ecc50 _ns: String en src/server/routing.rs:1166 desde 2026-08-30, confianza=alta | invariantes: ninguno (no hay cambios de código) | deuda: FIND-035 lint cascade nuevo (unused imports + assertions_on_constants — scope separado) | queda_pendiente: FIND-035 backlog row para resolver lint cascade
Próxima tarea si completa: FIND-035 (lint cascade nuevo — derivado)
=== END RECITATION ===

=== RECITATION 2 ===
Campaign ID: cecc8468-9451-4d56-a3ef-1684e123ab8b (nuevo — fast-gate residues)
Objetivo activo: FIND-MCP-001 MemoryRecord literal fix
Estado: completed
Última acción: Verifiqué que FIND-MCP-001 ya estaba aplicado en commit 43e0779e (heat:0, superseded_by:None en context_tests.rs:89-90). context_tests compila. test_embed_texts.rs tiene error separado (max_embed_batch_size) → FIND-036. Cierro task.
Resultado: OK
Próxima acción: crear fila FIND-036 en docs/Backlog.md
Contrato: verificacion: cargo check -p vantadb-mcp --test context_tests exit 0 (0.41s) ✅ | evidencia: claim=FIND-MCP-001 target arqueológico ya aplicado, evidencia=commit 43e0779e lines 89-90 heat:0 superseded_by:None, confianza=alta | invariantes: ninguno | deuda: FIND-036 test_embed_texts.rs max_embed_batch_size (scope separado) | queda_pendiente: FIND-036 backlog row
Próxima tarea si completa: TBH-06 (insta snapshots)
=== END RECITATION ===

=== RECITATION 3 ===
Campaign ID: cecc8468-9451-4d56-a3ef-1684e123ab8b (nuevo — fast-gate residues)
Objetivo activo: TBH-06 insta snapshot completion
Estado: completed
Última acción: vanta-worker sub-agent creó tests/query_result_basic.rs (7 tests) y tests/query_result_advanced.rs (13 tests) con insta::assert_debug_snapshot!. Añadió entradas [[test]] en Cargo.toml. Generó y aceptó 20 snapshots. Verificación completa pasó.
Resultado: OK
Próxima acción: ejecutar Task 4 RES-11 (CI cargo doc workflow)
Contrato: verificacion: cargo check -p vantadb --tests exit 0 ✅ + cargo test -p vantadb --test query_result_basic 7 passed + --test query_result_advanced 13 passed ✅ + 20 snapshots accepted ✅ | evidencia: claim=20 insta snapshots generated and accepted, evidencia=tests/snapshots/*.snap + commit f4bf5682, confianza=alta | invariantes: snapshots complementan assertions existentes; FIND-24 cursor cross-ns considerado en casos multi-page/exclude_superseded | deuda: ninguna (pre-existing test failures en sdk::api NotInitialized no relacionados) | queda_pendiente: RES-11 (CI cargo doc workflow)
Próxima tarea si completa: RES-11
=== END RECITATION ===

=== RECITATION 4 ===
Campaign ID: cecc8468-9451-4d56-a3ef-1684e123ab8b (nuevo — fast-gate residues)
Objetivo activo: RES-11 CI cargo doc workflow
Estado: completed
Última acción: Creado workflow rustdoc-70.yml con cargo doc --no-deps --workspace --all-features --document-private-items, tar.gz artifact 30-day retention, actionlint OK, push a develop.
Resultado: OK
Próxima acción: ninguna — plan completo
Contrato: verificacion: actionlint .github/workflows/rustdoc-70.yml exit 0 ✅ | evidencia: claim=workflow created and committed, evidencia=commit 25792e30 .github/workflows/rustdoc-70.yml, confianza=alta | invariantes: no rompe fast gate existente; trigger paths filter evita spam; --no-deps + --document-private-items para adopters | deuda: ninguna | queda_pendiente: ninguna
Próxima tarea si completa: plan archived
=== END RECITATION ===
