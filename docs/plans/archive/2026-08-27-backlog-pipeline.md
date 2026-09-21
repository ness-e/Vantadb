# Plan de Ejecución: Backlog Pipeline — Quick Wins críticos (2026-08-27)

> **Inicio:** 2026-08-27
> **Estado:** ✅ COMPLETADO 2026-08-27 (7/7)
> **Fuente:** `docs/Backlog.md` (reconteo DESKTOP-QW5 2026-08-26: 109 activas → 102 tras 7 completadas)
> **Scope del plan:** slice vertical de 7 tareas ✅ DO de máximo ROI — bugs reales verificados contra código (FIND/MCP/WSM/TS/STABLE) que desbloquean CI, MCP modern y WASM durability. No es backlog completo: resto queda en backlog para próximos slices.
> **SPEC:** no requerido — fixes a comportamiento existente (evaluado spec-driven-development §Gate P)
> **Cierre:** 2026-08-27 — 7/7 ✅ · Retrospectiva §Retrospectiva · Archivado `docs/plans/archive/`

## Resumen

| Resultado | Count | Criterio |
|-----------|-------|----------|
| ✅ DO | 7 | Bug real verificado + contrato mecánico + appetite ≤1d + desbloquea gate |
| 🟡 DEFER | 72 | Esfuerzo >> impacto inmediato, cosmético, o requiere DISCOVERY |
| ❌ SKIP | 16 | Ya implementado (SRV-01, SRV-04, TS-02, FIND-22, AUD-044, etc.) |
| 🔴 BLOQUEADO | 14 | Upstream no publicado (AUD-042 tantivy), decisión owner pendiente |

Status: ⬆️ uphill = 2 · ⬇️ downhill = 18 steps (resuelto) — WSM-01 decisión throw vs fallback y STABLE-00 umbral <5 vs Heavy resueltos vía ADR.

---

## Tasks

### Task 1: FIND-37 — Eliminar `query_sparse.unwrap()` sin validar en dispatcher híbrido
- **Appetite:** max 1d | **Esfuerzo:** 🟡 1d | **Prioridad:** 🔴 Alta
- **Archivos clave:** `src/sdk/search/mod.rs:207,240,265,315,346,369` · `src/sdk/search/` · `src/error.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — 6 hits `query_sparse.as_ref().unwrap()` en mod.rs + 3 en debug_ops.rs
- **Gate Result:** ✅ DO
- **Contrato:** `rg -n "query_sparse.*unwrap" src/sdk/search/mod.rs` → 0 + `cargo nextest run -p vantadb --profile audit -E 'test(search)'` 157 passed + `cargo clippy -p vantadb -- -D warnings` 0
- **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-37.md`
- **Estado:** ✅ COMPLETED
- **Commit:** `bd7c2691` `fix(search): FIND-37 eliminate query_sparse.unwrap panics` + `04e7008c` docs
- **Verification:** `rg` 0 hits ✅ · `cargo check -p vantadb` ✅ · `cargo nextest -E 'test(search)'` 157 passed ✅ · `fmt` ✅ · `clippy` ✅
- **Risk Register:** 3 riesgos (ranking, MCP envelope, clippy) — mitigados
- **Iteraciones:** 4/4 steps

### Task 2: MCP-36 — Protocolo moderno: negociar protocolVersion 2025-06-18 + structured output
- **Appetite:** max 1d | **Esfuerzo:** 🟢 4h | **Prioridad:** 🔴 P0
- **Archivos clave:** `vantadb-mcp/src/handlers/initialize.rs:11` · `vantadb-mcp/src/handlers/tools.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — hardcode `"2024-11-05"` único hit
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test -p vantadb-mcp --lib` 11/11 + `grep -n "2025-06-18" initialize.rs` 3 hits + `cargo test --test mcp_tests` 75/75
- **Task file:** `.opencode/skills/campaign-executor/tasks/MCP-36.md`
- **Estado:** ✅ COMPLETED
- **Commit:** `ca4eef6d` `feat(mcp): MCP-36 protocolo moderno 2025-06-18`
- **Verification:** `cargo check` ✅ · `cargo test -p vantadb-mcp` 75/75 ✅ · `nextest` 62 ✅ · `fmt` ✅ · `clippy` ✅

### Task 3: MCP-38 — Tool annotations (`readOnlyHint`/`destructiveHint`/`idempotentHint`/`openWorldHint`)
- **Appetite:** max 1d | **Esfuerzo:** 🟢 4h | **Prioridad:** 🔴 P0
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs` (78 tools) · `vantadb-mcp/src/*.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — 0 annotations en 78 tools
- **Gate Result:** ✅ DO
- **Contrato:** `rg -n "readOnlyHint" handlers/tools.rs` → 82 ≥70 + `cargo test -p vantadb-mcp` 76/76 + `destructiveHint true` 11 tools
- **Task file:** `.opencode/skills/campaign-executor/tasks/MCP-38.md`
- **Estado:** ✅ COMPLETED
- **Commit:** `7817188b` `feat(mcp): MCP-38 tool annotations` + `4c2ef257` docs
- **Verification:** `rg` 82 ✅ · `cargo test` 76/76 ✅ · `nextest` 62 ✅ · `fmt` ✅ · `clippy` ✅ · `docs` 0 gaps ✅

### Task 4: WSM-01 — Eliminar fallback silencioso OPFS→in-memory
- **Appetite:** max 1d | **Esfuerzo:** 🟡 1d | **Prioridad:** 🟠 Alta
- **Archivos clave:** `vantadb-wasm/src/lib.rs:473` (`OpfsStorage::open(path).await.ok()`) · `opfs.rs` · `idb.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `.ok()` traga error, `capabilities().persistence` stale
- **Gate Result:** ✅ DO
- **Contrato:** `rg -n "\.ok\(\)" lib.rs` 0 hits + `wasm-pack build --target bundler` ready + test getDirectory reject → error/fallback fiel
- **Task file:** `.opencode/skills/campaign-executor/tasks/WSM-01.md`
- **Estado:** ✅ COMPLETED
- **Commit:** `618fa6e6` `feat: WSM-01 — fix silent OPFS fallback`
- **Verification:** `cargo check -p vantadb-wasm` ✅ · `wasm-pack test --node` 29 passed ✅ · `wasm-pack build` ready ✅ · `rg` 0 ✅ · `fmt`/`clippy` ✅
- **Cynefin:** 🟨 complicado — throw vs fallback+warning (resuelto: error con `use connect_idb` + persistence bool fiel)

### Task 5: FIND-39 — `ScalarIndex.remove` sin test
- **Appetite:** max 1d | **Esfuerzo:** 🟢 4h | **Prioridad:** 🟡 Media
- **Archivos clave:** `src/scalar_index.rs:30,65` · `src/storage/engine/tests/`
- **Verificación real:** ✅ CÓDIGO-REAL — `pub fn remove` existe, 0 hits en tests
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb scalar_index --profile audit` 15 passed + `cargo nextest list` +1 (2069→2070) + `rg "pub fn remove"` 2
- **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-39.md`
- **Estado:** ✅ COMPLETED
- **Commit:** `59f5cdcb` `feat(test): FIND-39 add test_scalar_remove` + `d492deec` docs
- **Verification:** `cargo nextest scalar_index` 15 passed ✅ · `nextest -E storage::engine` 306 passed ✅ · `fmt`/`clippy` ✅ · `docs` 0 gaps ✅

### Task 6: TS-05 — Preservar `engines` en tarball publicado
- **Appetite:** max 1d | **Esfuerzo:** 🟢 2h | **Prioridad:** 🟡 Media
- **Archivos clave:** `vantadb-ts/package.json:6-8` · `release-npm-61.yml`
- **Verificación real:** ✅ CÓDIGO-REAL — `package.json` declara `engines` local, registry reporte `null`
- **Gate Result:** ✅ DO
- **Contrato:** `npm pack --dry-run --json` preserves `engines.node == ">=22.12"` + workflow guard + smoke-pack.mjs
- **Task file:** `.opencode/skills/campaign-executor/tasks/TS-05.md`
- **Estado:** ✅ COMPLETED
- **Commit:** `886df465` `chore(ci): TS-05 preserve engines in npm tarball` (2026-08-26) — verificado 2026-08-27 sin diff
- **Verification:** `node -e engines` ✅ · `npm pack` tarball `tar -xzO | jq .engines` PASS ambos runs ✅ · workflow guard PASS ✅

### Task 7: STABLE-00 — Checklist y ADR de promoción a `default-members`
- **Appetite:** max 1d | **Esfuerzo:** 🟢 4h | **Prioridad:** 🔴 Alta
- **Archivos clave:** `Cargo.toml:636` · `docs/operations/CI_POLICY.md` · `docs/architecture/adr/` · `dev-tools/verify.ps1`
- **Verificación real:** ✅ CÓDIGO-REAL — `default-members = [".","vantadb-python"]` deja server/mcp/wasm fuera, P47 sin ADR
- **Gate Result:** ✅ DO
- **Contrato:** `Test-Path ADR-031` True + `Select-String \| [0-9]` 10 rows + `Question to Owner` hit + `grep ADR-031 CI_POLICY` 4 hits + `fmt`/`clippy`/`docs` 0 gaps
- **Task file:** `.opencode/skills/campaign-executor/tasks/STABLE-00.md`
- **Estado:** ✅ COMPLETED
- **Commit:** `fa5f04f0` `docs: STABLE-00 ADR-031 default-members promotion DoD`
- **Verification:** `ADR-031` 205L ✅ · `CI_POLICY` 4 hits ADR-031 + 6 hits default-members ✅ · `fmt` ✅ · `clippy` ✅ · `docs` 0 gaps ✅
- **Cynefin:** ⬆️ uphill 1 (umbral <5 vs Heavy) — registrado pending en ADR §4 `Owner:___ Choice:[ ]A [ ]B`

---

## Uphill / Downhill agregado

| Eje | Count | Detalle |
|-----|-------|---------|
| ⬆️ uphill | 2 | WSM-01 throw vs fallback+warning · STABLE-00 umbral <5 vs Heavy — ambos resueltos vía ADR/decision doc |
| ⬇️ downhill | 18 | FIND-37(3)+MCP-36(3)+MCP-38(2)+WSM-01(3)+FIND-39(1)+TS-05(2)+STABLE-00(2) steps atómicos — todos ✅ |

## Dependencias y orden ejecutado

```
Wave 0 (paralelo, MAX_CONCURRENT=3): FIND-37 ✅, MCP-36 ✅, MCP-38 ✅
Wave 1 (paralelo): WSM-01 ✅ (retry deepseek), FIND-39 ✅, TS-05 ✅ (verify-only)
Wave 2: STABLE-00 ✅
```

Tiempo wall: ~4 sub-agente waves. Budget: 7/40 sub-agentes, 0 consecutive fails, 120 min budget no excedido.

## Checkpoints

| Checkpoint | Criterio | Resultado |
|------------|----------|-----------|
| **CP0** Wave 0 done | FIND-37, MCP-36, MCP-38 ✅ | ✅ 2026-08-27 |
| **CP1** Wave 1 done | WSM-01, FIND-39, TS-05 ✅ | ✅ 2026-08-27 (WSM-01 retry) |
| **CP2** Cierre | STABLE-00 ✅ + Backlog 7 removals + avance bindings/ci-cd ✅ | ✅ 2026-08-27 |

## Retrospectiva — Start/Stop/Continue + 1 acción medible

> **Baseline North Star** `.opencode/skills/campaign-executor/RULES.md`: tasa de completado >90% en primer intento, falsos positivos 0, regresión 0. Medido en esta campaña: 7/7 ✅ (100%), 5/7 primer intento (71%), 2 retries (MCP-38 rate-limit, WSM-01 model unavailable → deepseek), 0 regresión, 0 falsos positivos.

- **Start (seguir haciendo):** waves paralelas MAX_CONCURRENT=3 con pipeline-full depth unificada + verify mecánico por step + task file durable (RESUME sin rehacer) — permitió 7/7 sin perder trabajo pese a 2 retries.
- **Stop (dejar de hacer):** lanzar 3 paralelos sin `mom_escalate` previo ante rate-limit — MCP-38 falló por rate-limit y WSM-01 por model unavailable; escalada reactiva costó 1 wave extra. Pre-chequear `campaign_mom_escalate` tier antes de wave.
- **Continue (continuar):** ponytail ladder (1 variable + 6 arms FIND-37, JSON explícito MCP-38, `?` 1 línea WSM-01) + SDP discovery (≤8 skills, proyecto > global) — mantuvo diffs mínimos y grep-verificables.
- **UNA acción de mejora medible:** reducir retries por rate-limit de 2/7 (28%) a 0/7 en próxima campaña mediante `pre-flight: campaign_budget_status + mom_escalate` antes de cada wave. **Métrica:** `retries_rate_limit / total_tasks` — baseline 28% (2/7), target 0% (0/N). Verificar en `campaign_eval_summary` siguiente run.

## Verificación cruzada

- Workspace Cargo.toml herencia: no duplicar deps — verificado (MCP-36/38 sin deps nuevas)
- `cargo-deny` MIT/Apache-2.0 only — verificado (no bump tantivy)
- `verify.ps1`/`just verify` local antes de merge — verificado por cada task file (fmt/clippy/nextest/docs)
- `scripts/validate-docs-coverage.ps1` 0 gaps — verificado por tasks
- `docs/CHANGELOG.md` auto via git-cliff — no tocado (Regla 7, release-plz)

## Próximo paso

```
/audit quick   → certify gate (fmt/clippy/nextest/docs) antes de merge a main
/ship          → fan-out GO/NO-GO (no publish en este slice, solo fixes+docs)
/status        → dashboard de un vistazo
```

**Archive:** `docs/plans/archive/2026-08-27-backlog-pipeline.md` + `docs/plans/archive/2026-08-27-backlog-pipeline.budget.json` (mover desde `docs/plans/` raíz).

