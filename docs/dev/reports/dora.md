# DORA Flow Metrics Report

> Generado por `dev-tools/dora.mjs` (P3-07) — 2026-10-30T21:11:37.073Z
> Fuentes: `docs/dev/plans/*.md` (0 tareas en 0 planes) + task files en `.opencode/skills/campaign-executor/tasks/` (0) + `*.budget.json` (0 con timestamps) + `verify-log.jsonl` (680 intentos de verify)
> ⚠️ **Fechas derivadas best-effort, NO normalizadas**. Prioridad: markers escritos (`**Inicio:**`, `**Estado:** COMPLETADO (fecha)`, `**Fecha:**`, `**Creado:**`, fechas en bloque de tarea) -> budget epoch ms (`startTime`/`lastActivity`) -> **file mtime**. Donde se usó mtime se marca `(mtime)`. Esto es exactamente lo que P2-05 (traceId por tarea) va a resolver: con traceId real, cada task tendrá timestamps estructurados.


## 1. Cycle / Lead time

| Métrica | Definición pragmática |
|---|---|
| **Lead time** | completado − request. Request = plan `**Inicio:**` del plan que la contiene, fallback `**Creado:**` del task file, fallback budget `startTime`, fallback mtime. |
| **Cycle time** | completado − startOfWork. Start = budget `startTime` (timestamps reales) → `**Creado:**`/`**Inicio:**` del task file → plan inicio. Sin budget ni Creado, cycle == lead (mismo origen). |

### Por tipo de tarea

| Tipo | Tasks | Completed | Lead avg (días) | Cycle avg (días) |
|---|---|---|---|---|
| — | 0 | 0 | — | — |

### Por tarea (completadas con fechas derivables)

| Task | Tipo | Plan | Request | Start | Completed | Lead | Cycle |
|---|---|---|---|---|---|---|---|
| _(sin tareas completadas con fechas aún)_ | | | | | | | |

> Nota: 0 tareas completadas sin fechas derivables quedan fuera de la tabla (estado ✅ sin fecha escrita ni mtime utilizable).

## 2. CFR (Change Failure Rate)

| Intentos de verify | Fallos | CFR |
|---|---|---|
| 680 | 202 | **29.7%** |

> Detalle de fallos por tarea: ?(1), ?(1), ?(-1), ?(-1), ?(-1), ?(1), ?(-1), FIND-54(1), MKT-18i(-1), ?(1), 7(-1), FIND-64(-1), BND-12(-1), ?(-1), ?(-1), ?(1), UX-19(1), ?(-1), 4(1), FIND-50(1), ?(-1), ?(1), MEM-66(1), 4(-1), ?(1), ?(-1), 3(1), AST-001(1), ?(-1), ?(-1), ?(-1), ?(-1), ?(-1), ?(1), AST-011(-1), ?(1), FIND-90(-1), FIND-88(1), FIND-63(-1), ?(-1), FIND-70(-1), ?(1), ?(-1), ?(-1), ?(-1), ?(-1), ?(1), ?(1), ?(-1), ?(-1), FIND-105(-1), FIND-110(-1), FIND-114(-1), ?(-1), IMPL-112-S2(-1), ?(-1), ?(-1), ?(-1), ?(-1), R-03(1), ?(-1), ?(1), ?(1), C-07(1), C-10(1), ?(-1), ?(1), C-07(-1), C-10(1), ?(1), C-07(-1), API-01(1), API-01(1), API-01(-1), API-01(1), ?(1), API-01(1), API-01(1), API-01(1), API-03(-1), API-05(1), API-05(1), API-06(-1), ?(-1), ?(-1), ?(-1), API-07(-1), ?(1), ?(1), HARD-01(-1), ?(-1), HARD-01(-1), HARD-01(1), HARD-01(1), ?(1), HARD-01(-1), HARD-04(-1), HARD-05(-1), HARD-05(-1), HARD-05(-1), HARD-06(-1), HARD-06(1), HARD-06(-1), HARD-07(1), HARD-07(1), HARD-07(1), HARD-07(-1), HARD-07(-1), HARD-07(-1), WIRE-10(-1), DEF-02(-1), DEF-02(0), WIRE-10(1), DEF-01(-1), DEF-01(1), DEF-03(-1), DEF-03(1), DEF-05(-1), DEF-03(-1), DEF-03(-1), ?(-1), DEF-08(-1), DEF-07(-1), WIRE-03(-1), ?(-1), ?(1), ?(-1), ?(-1), ?(-1), WIRE-07(1), WIRE-07(1), WIRE-07(1), WIRE-07(-1), WIRE-07(-1), ?(-1), ?(-1), WIRE-08(-1), ?(-1), MGR-13(-1), MGR-13(1), ?(-1), ?(-1), ?(-1), ?(-1), SCH-02(-1), SCH-02(1), SCH-02(-1), SCH-02(-1), SCH-02(1), ?(-1), ?(-1), ?(-1), ?(-1), FIND-226(1), FIND-227(-1), FIND-231(-1), FIND-231(1), FIND-235(-1), FIND-233(-1), DX-01(-1), DX-01(1), DUR-01(-1), DUR-03(-1), DUR-03(1), DUR-03(1), DUR-03(1), DUR-03(1), BENCH-01(-1), DIST-01(-1), ?(-1), ?(1), DIST-02(-1), ?(-1), 72(-1), 72(-1), 23(-1), 24(-1), 27(-1), 27(1), ?(-1), 33(-1), 33(-1), ?(-1), ?(1), ?(-1), ?(1), 39(-1), 41(-1), 41(-1), 42(-1), 42(-1), 48(1), 49(-1), 49(-1), 48(-1), 50(1), 60(-1), 60(1), MEMG-19(-1), 65(-1), 70(-1), 71(-1)

## 3. Recovery Time

| Task | Fallo (exit) | Comando | Clasificación | Δt |
|---|---|---|---|---|
| MKT-18i | -1 | `rg -ci "ollama|anythingllm|anything-llm" docker-compose.yml` | no-ejecutado (espurio) | 1.7s |
| 7 | -1 | `cargo fmt --check -p vantadb-mcp` | no-ejecutado (espurio) | 5.5s |
| FIND-64 | -1 | `python -m pytest integrations/llamaindex/tests -q` | no-ejecutado (espurio) | 180.81h |
| AST-001 | 1 | `Test-Path scripts/anti_stutter_map.json -and (Test-Path docs` | fallo real | 11.6s |
| R-03 | 1 | `npx tsc --noEmit` | fallo real | 15.9s |
| C-07 | 1 | `pwsh -NoProfile -Command "$n = (git ls-remote --heads origin` | fallo real | 39.9s |
| C-10 | 1 | `python "C:/Users/Eros/AppData/Local/Temp/opencode/c10-verify` | fallo real | 39.6s |
| C-07 | -1 | `python "C:/Users/Eros/AppData/Local/Temp/opencode/c07-verify` | no-ejecutado (espurio) | 7min |
| C-10 | 1 | `python "C:/Users/Eros/AppData/Local/Temp/opencode/c10-verify` | fallo real | 5min |
| C-07 | -1 | `python "C:/Users/Eros/AppData/Local/Temp/opencode/c07-verify` | no-ejecutado (espurio) | 3min |
| API-01 | 1 | `$env:CARGO_TARGET_DIR='target\session-api01'; $env:PYO3_PYTH` | fallo real | 50.3s |
| API-01 | 1 | `Set-Location vantadb-ts; npx --no-install vitest run tests/w` | fallo real | 2min |
| API-01 | -1 | `node dev-tools/build-wasm-types.mjs; exit $LASTEXITCODE` | no-ejecutado (espurio) | 2.3s |
| API-01 | 1 | `$env:CARGO_TARGET_DIR='target\session-api01'; pwsh -NoProfil` | fallo real | 1min |
| API-01 | 1 | `$env:CARGO_TARGET_DIR='target\session-api01'; cargo test --t` | fallo real | 50.3s |
| API-01 | 1 | `$env:CARGO_TARGET_DIR='target\session-api01'; pwsh -NoProfil` | fallo real | 1min |
| API-03 | -1 | `node scripts/check_openapi_parity.mjs` | no-ejecutado (espurio) | 6.7s |
| API-06 | -1 | `cargo test --target-dir target/session-api01 -p vantadb --li` | no-ejecutado (espurio) | 57.9s |
| API-07 | -1 | `cargo fmt --all -- --check` | no-ejecutado (espurio) | 14.0s |
| HARD-01 | -1 | `cargo semver-checks check-release` | no-ejecutado (espurio) | 59min |
| HARD-01 | -1 | `cargo semver-checks --version` | no-ejecutado (espurio) | 10.9s |
| HARD-01 | 1 | `pwsh -NoProfile -File target/session-api01/verify-check-rele` | fallo real | 32min |
| HARD-01 | 1 | `cargo semver-checks check-release` | fallo real | 28min |
| HARD-01 | -1 | `pwsh -NoProfile -File target/session-api01/verify-docs.ps1` | no-ejecutado (espurio) | 16.8s |
| HARD-04 | -1 | `rg "vanta[_-]memory" vantadb-python vantadb-ts vantadb-node ` | no-ejecutado (espurio) | 25.1s |
| HARD-05 | -1 | `rg -n "sin -p|NUNCA .*cargo (test|nextest)" .opencode/AGENTS` | no-ejecutado (espurio) | 2.1s |
| HARD-05 | -1 | `rg -n "sin -p|NUNCA .*cargo (test|nextest)" .opencode/AGENTS` | no-ejecutado (espurio) | 36.4s |
| HARD-05 | -1 | `rg -n "8080|unificaci[oó]n de features|f6c395ef" docs/dev/re` | no-ejecutado (espurio) | 57.7s |
| HARD-06 | -1 | `pwsh -NoProfile -Command "pwsh scripts/validate-docs-coverag` | no-ejecutado (espurio) | 1.7s |
| HARD-06 | 1 | `pwsh -NoProfile -Command "$t=0; foreach ($p in 'dream','appr` | fallo real | 29.3s |
| HARD-06 | -1 | `pwsh -NoProfile -Command "if ((Select-String -Path .opencode` | no-ejecutado (espurio) | 1.8s |
| HARD-07 | 1 | `node --test .opencode/task-system/mcp/state-persistence.test` | fallo real | 56.4s |
| HARD-07 | 1 | `bun test .opencode/task-system/mcp` | fallo real | 16.7s |
| HARD-07 | 1 | `node --test .opencode/task-system/mcp/review-gate.test.mjs` | fallo real | 29.3s |
| HARD-07 | -1 | `node "C:/Users/Eros/AppData/Local/Temp/opencode/hard07-e2e-m` | no-ejecutado (espurio) | 20min |
| HARD-07 | -1 | `node --test .opencode/task-system/mcp/review-gate.test.mjs` | no-ejecutado (espurio) | 9.0s |
| HARD-07 | -1 | `node .opencode/task-system/config/parity-check.mjs` | no-ejecutado (espurio) | 2.0s |
| WIRE-10 | -1 | `cargo fmt --check` | no-ejecutado (espurio) | 23.0s |
| DEF-02 | -1 | `rg "v0\.1\.x" docs/user/operations/EXPERIMENTAL_FEATURES.md` | no-ejecutado (espurio) | 0.9s |
| DEF-02 | 0 | `rg "v0\.1\.x" docs/user/operations/EXPERIMENTAL_FEATURES.md` | fallo real | 1.0s |
| WIRE-10 | 1 | `pwsh -NoProfile -File dev-tools/verify.ps1` | fallo real | 8.0s |
| DEF-01 | -1 | `rg -n "three entry points" README.md docs/dev/vision/VISION.` | no-ejecutado (espurio) | 1.1s |
| DEF-03 | -1 | `actionlint .github/workflows/gate-docs.yml` | no-ejecutado (espurio) | 2min |
| DEF-03 | 1 | `"C:\Users\Eros\AppData\Local\Microsoft\WinGet\Packages\rhysd` | fallo real | 40.3s |
| DEF-05 | -1 | `rg -n "North Star" SPEC.md docs/dev/vision/VISION.md` | no-ejecutado (espurio) | 3.6s |
| DEF-03 | -1 | `& 'C:\Users\Eros\AppData\Local\Microsoft\WinGet\Packages\rhy` | no-ejecutado (espurio) | 1.09h |
| DEF-03 | -1 | `pwsh -NoProfile -File scripts/validate-frontier.ps1` | no-ejecutado (espurio) | 4min |
| DEF-08 | -1 | `rg -n "SLO|primer recall|first recall" docs/dev/strategy/DIS` | no-ejecutado (espurio) | 1.1s |
| DEF-07 | -1 | `rg -n "core-promise" SPEC.md docs/user/operations/EXPERIMENT` | no-ejecutado (espurio) | 22.3s |
| WIRE-03 | -1 | `& vantadb-python/.venv/Scripts/python.exe -m pytest vantadb-` | no-ejecutado (espurio) | 21.0s |
| WIRE-07 | 1 | `pwsh -NoProfile -Command "$env:CARGO_BUILD_JOBS=2; cargo che` | fallo real | 2min |
| WIRE-07 | 1 | `pwsh -NoProfile -Command "$t = (rustup target list --install` | fallo real | 21.0s |
| WIRE-07 | 1 | `pwsh -NoProfile target/session-wire07/unit-graph-features.ps` | fallo real | 57.5s |
| WIRE-07 | -1 | `pwsh -NoProfile -Command "cargo semver-checks -p vantadb 2>&` | no-ejecutado (espurio) | 14min |
| WIRE-08 | -1 | `cargo fmt --all --check` | no-ejecutado (espurio) | 1.6s |
| SCH-02 | -1 | `cargo fmt --all -- --check` | no-ejecutado (espurio) | 24min |
| SCH-02 | 1 | `cargo nextest run --profile audit -p vantadb --build-jobs 2` | fallo real | 12min |
| SCH-02 | -1 | `cargo fmt --all --check` | no-ejecutado (espurio) | 8min |
| FIND-227 | -1 | `actionlint .github/workflows/ci-rust.yml` | no-ejecutado (espurio) | 2.5s |
| FIND-231 | -1 | `actionlint .github/workflows/ci-rust.yml` | no-ejecutado (espurio) | 2min |
| FIND-231 | 1 | `"C:\Users\Eros\AppData\Local\Microsoft\WinGet\Packages\rhysd` | fallo real | 13.3s |
| FIND-233 | -1 | `python benchmarks/compare_baseline.py --self-test` | no-ejecutado (espurio) | 1.0s |
| DX-01 | -1 | `cd vantadb-ts && npx vitest run` | no-ejecutado (espurio) | 7.1s |
| DX-01 | 1 | `cd vantadb-ts && "C:\Users\Eros\AppData\Local\Temp\opencode\` | fallo real | 1.4s |
| DUR-01 | -1 | `cargo fmt --check` | no-ejecutado (espurio) | 5.1s |
| DUR-03 | -1 | `cargo nextest run --profile audit -p vantadb --test edge_cas` | no-ejecutado (espurio) | 1min |
| DUR-03 | 1 | `cargo nextest run --profile audit -p vantadb --test edge_cas` | fallo real | 5min |
| DUR-03 | 1 | `cargo nextest run --profile audit -p vantadb --test edge_cas` | fallo real | 5min |
| DUR-03 | 1 | `cargo nextest run --profile audit -p vantadb --test edge_cas` | fallo real | 5min |
| DUR-03 | 1 | `cargo nextest run --profile audit -p vantadb --test edge_cas` | fallo real | 4min |
| BENCH-01 | -1 | `.venv/Scripts/python.exe -m py_compile benchmarks/competitiv` | no-ejecutado (espurio) | 4.8s |
| DIST-01 | -1 | `cargo publish --dry-run -p vanta-memory --allow-dirty` | no-ejecutado (espurio) | 27.6s |
| DIST-02 | -1 | `target\dist02-venv\Scripts\python -m pytest vantadb-python/t` | no-ejecutado (espurio) | 1.7s |
| 72 | -1 | `cargo fmt --check` | no-ejecutado (espurio) | 29.8s |
| 27 | -1 | `node scripts/docs/check-links.mjs` | no-ejecutado (espurio) | 4.1s |
| 33 | -1 | `node scripts/docs/check-links.mjs` | no-ejecutado (espurio) | 3.2s |
| 33 | -1 | `rg -n includeUpdaterJson docs/` | no-ejecutado (espurio) | 1.0s |
| 41 | -1 | `cargo fmt --all -- --check` | no-ejecutado (espurio) | 1.9s |
| 41 | -1 | `cargo nextest run --profile audit -p vanta-memory --test l1_` | no-ejecutado (espurio) | 13.4s |
| 42 | -1 | `cargo fmt --check` | no-ejecutado (espurio) | 14.8s |
| 42 | -1 | `cargo clippy -p vanta-memory --all-targets --all-features --` | no-ejecutado (espurio) | 2min |
| 48 | 1 | `node scripts/docs/gen-index.mjs --check` | fallo real | 1.9s |
| 48 | -1 | `pwsh scripts/check-avance-coverage.ps1` | no-ejecutado (espurio) | 2.2s |
| 60 | -1 | `node scripts/docs/check-links.mjs` | no-ejecutado (espurio) | 5.0s |
| 60 | 1 | `node scripts/docs/gen-index.mjs --check` | fallo real | 2.0s |
| MEMG-19 | -1 | `node scripts/docs/check-links.mjs` | no-ejecutado (espurio) | 4.4s |
| 65 | -1 | `node scripts/docs/check-links.mjs` | no-ejecutado (espurio) | 3.7s |
| 70 | -1 | `cargo check -p vantadb --features server` | no-ejecutado (espurio) | 41.9s |
| 71 | -1 | `node scripts/docs/check-links.mjs` | no-ejecutado (espurio) | 2.6s |

**Δt promedio (solo fallos reales, exit ≠ -1):** 0.1h sobre 31 pares (58 espurios excluidos del promedio).


> Caveat: entradas con `taskId:null` (206 en el log actual) no son pareables — quedan fuera de esta métrica.

## 4. Throughput

| Periodo (días) | Tareas completadas |
|---|---|
| Últimos 7 | 0 |
| Últimos 30 | 0 |

> ⚠️ Sin tareas ✅ COMPLETED detectadas con fecha de completado — throughput 0.

## 5. Flow table

### Por tipo de tarea

| Tipo | Total | Pending | In-progress | Completed | Failed | Unknown |
|---|---|---|---|---|---|---|
| — | 0 | 0 | 0 | 0 | 0 | 0 |

### Por plan file

| Plan | Total | Pending | In-progress | Completed | Failed | Lead avg (días) |
|---|---|---|---|---|---|---|
| _(sin plan files)_ | | | | | | |

### Sin plan file (task files sueltos)

| Bucket | Total | No completadas | Completed |
|---|---|---|---|
| task:flat | 0 | 0 | 0 |
| task:complete | 0 | 0 | 0 |
| task:closed | 0 | 0 | 0 |

## 6. Limitaciones

- **Fechas no estructuradas**: los plan files no tienen un campo de timestamp por tarea normalizado; las fechas se extraen de markers ad-hoc (varían entre `✅ COMPLETADO (2026-08-10)`, `**Estado: completed`, `**Fecha:**`, ISO en task files, epoch ms en budget). Donde no hay marker se usó **file mtime** → esas filas son aproximaciones del día en que el archivo se tocó, no el día real del evento.
- **Cycle vs Lead**: sin budget `startTime` ni `**Creado:**`, startOfWork cae al plan `**Inicio:**` y cycle == lead. Los avgs por tipo mezclan ambas calidades.
- **CFR**: con `verify-log.jsonl` vacío (0 líneas) el 0% es baseline sin telemetría; no es evidencia de ausencia de fallos.
- **Throughput**: cuenta tareas ✅ COMPLETED con fecha de completado derivada; tareas completadas sin fecha quedan fuera.
- **Numeric budget keys**: claves numéricas de budget.json se resolvieron vía mapeo `Task N → id` del plan; si no hay match quedan como id numérico (tarea desconocida, cuenta como `unknown`).
- **Recovery time**: el emparejamiento fail→pass usa solo `taskId` (no `taskId+command`) porque los reintentos cambian flags entre intentos; un par con fallo `exitCode:-1` es "no-ejecutado" (timeout/kill, espurio), no evidencia de recovery real. Entradas sin taskId no son pareables.
- **P2-05 traceId**: con traceId por tarea, plan files deberán persistir `createdAt`/`completedAt` estructurados; este reporte se recalculará sin fallback mtime.
- **Cobertura de task files**: los plan files P0/P1/P2/P3 no tienen todavía task files propios en `tasks/` (referencian `skills/campaign-executor/tasks/P*-NN.md` que aún no existen) — sus estados vienen del plan; los task files existentes (AUD/ERR/INV/COMP/DESKTOP/…) alimentan la vista de detalle.
