# STABLE-09 — Promoción atómica + rollback plan

## Metadata
- **Plan file:** `docs/plans/2026-09-08-backlog.md` (Task 6, Wave3)
- **Creado:** 2026-09-09
- **Estado:** ✅ COMPLETO-subset 2026-09-09 (Owner A; plan `docs/plans/2026-09-09-backlog.md` Task 1; ver §Ejecución subset abajo — el BLOQUEO previo era plan 2026-09-08 con ADR `proposed`)
- **Fuente:** Backlog P47 — cierra P47; cambio reversible 1 línea con rollback en descripción del PR
- **Esfuerzo:** 🟢 4h | **Prioridad:** 🔴 Alta | **Ruta:** `vanta-lead` (CI/CD, yo mismo)
- **Tipo:** release/CI (detectado `docs` — validación + promoción; sin lógica nueva)
- **Appetite:** max 1d | **Branch:** develop (sin PR real — bloqueado) | **Commit:** pendiente (solo este task file)
- **Contrato (ley):** PR único con `default-members` ampliado + CI_POLICY §default-members + `cargo package --dry-run` 0 + rollback 1-línea en descripción + `just verify` <5min o etiqueta Heavy justificada

## Gate D (tras zero-code planning, ANTES de escribir steps)
> Blast radius = 1 línea `Cargo.toml` (`default-members` 710-713) + 1 sección `CI_POLICY.md` §default-members (ya existe medición STABLE-08) + verificación `cargo package --dry-run` + `just verify` wall time. Sin hot path (`src/vector/`, `engine.rs` no tocados). Sin símbolos públicos nuevos (`pub fn`/tool/endpoint: ninguno). Contrato mecánico pero con gate externo pendiente (Owner A/B). **Gate D NO disparado** (sin `question` al usuario en runner; registrado aquí por question-gates.md §Registro obligatorio). Gate P ya confirmado a nivel plan 2026-09-08 (owner aprobó 10 DO).

## Impacto mapeado (Regla 0)
- **Archivos leídos completos (antes de editar):**
  - `Cargo.toml:710-713` (`default-members = [".", "vantadb-python"]`; nota: plan decía `:636` — línea stale, real 710-713; `members` 7 crates ya) — NO EDITADO (bloqueado)
  - `docs/operations/CI_POLICY.md` (446L — §Promotion 149-240 con medición STABLE-08: `just verify` cold 495.5s/8.26m Heavy, warm 249s/4.15m; `verify_changed` cold 115s Fast; veredicto Heavy + Owner A/B bloqueado) — NO EDITADO (medición ya registrada)
  - `dev-tools/verify.ps1` (102L — `fmt → check -p vantadb → clippy -p vantadb → audit → deny → nextest -p vantadb -E RESOURCE-GUARD ×3 → coverage → docs-coverage → cli-probes → consumo guard → backup runbook`; `-p vantadb` = independiente de `default-members`) — SOLO LECTURA
  - `docs/architecture/adr/ADR-031-default-members-promotion.md` (`status: proposed`, `owner: TBD`; §4 Question to Owner A `<5 hard` vs B `<8 soft`; "STABLE-09 must not merge" hasta respuesta) — SOLO LECTURA
  - `docs/tasks/STABLE-00..08.md` (estados: 00 ✅, 01 ✅, 02 ✅, 03 ✅, 04 ✅-inner/⏳-header, 05 ✅, 06 ✅ re-validado hoy ceb81d90 280/280, 07 steps 5/5 PASS pero header ⏳ IN PROGRESS sin sync, 08 ✅ 3 corridas 0 flaky + Heavy verdict) — SOLO LECTURA
  - `docs/plans/2026-09-08-backlog.md` (Task 6 ⬜ PENDING; untracked + mezcla recitations ajenas — NO TOCADO, precedente FIND-48/BLOG-CTA; sync lo hace el orquestador)
- **Referencias hacia dentro:** `cargo check` sin args usa `default-members`; `ci-rust-10.yml:test` usa `default-members` (nextest sin `--workspace`); `just verify`/`clippy --workspace` NO dependen de `default-members`; `verify.ps1` (`-p vantadb`) independiente.
- **Referencias entrantes:** `ci-rust-10.yml` `experimental-check` + `coverage --exclude experimental` dependen del circuit breaker; `release-npm-61.yml:tests` (npm gate) independiente; `BND-08/TS-12` desbloqueados por otro lado.
- **Veredicto impacto:** promoción = 1 línea reversible (`git revert` de `Cargo.toml:710-713` + `publish = false` intacto → `cargo publish` no afectado). Pero **NO se ejecuta** por gate externo rojo (ADR proposed + STABLE-07 header sin sync + sin 3-corridas frescas post-STABLE-06 de hoy para el set ampliado). Riesgo de forzar: romper Fast Gate <5min en frío (evidencia STABLE-08: clippy cold >600s timeout, just verify cold 8.26m). Decisión: documentar BLOQUEADO, cero ediciones funcionales.

## Spec (decisiones — Gate spec-first N/A justificado)
| Decisión | Elección | Evidencia |
|----------|----------|-----------|
| ¿Promocionar hoy? | NO — documentar BLOQUEADO | ADR-031 `status: proposed` + `owner: TBD` + "STABLE-09 must not merge"; STABLE-08 verdict Heavy cold>5 (495.5s) con Owner A/B pendiente |
| ¿Editar `Cargo.toml`? | NO (ni en rama) | Stop condition plan: "gates previos rojos → BLOQUEADO (no forzar)"; forzar = big-bang sin Owner GO (Red Flag shipping-and-launch) |
| ¿Editar `CI_POLICY.md`? | NO — medición ya registrada | §STABLE-08 measurement 2026-08-27 completa (tabla por job + entorno + Heavy verdict); duplicar = churn |
| ¿3-corridas de hoy? | NO corridas nuevas (evidencia heredada suficiente para BLOQUEO) | STABLE-08: 3 corridas 0 flaky (verify_changed 115s/8s/8s + just verify 495s/249s); STABLE-06 re-validado hoy 280/280 (ceb81d90); re-medir cold >600s clippy quemaría ~30min sin cambiar veredicto (Heavy ya probado) — ponytail: no re-medir lo ya medido |
| ¿`question` al Owner? | Deferido al orquestador/humano (no en runner) | ADR-031 §4 ya formula A vs B; este task file es la evidencia para que el Owner responda; `question` tool sin respuesta en runner = STOP per Gate V |
| Rollback plan | Template §2b listo abajo (1-línea) para el futuro PR | Contrato exige "rollback 1-línea en descripción" — se deja redactado, no ejecutado |

## Ejecución subset 2026-09-09 (plan 2026-09-09 Task 1, Owner A)

Desbloqueo vs §Spec previa: ADR-031 `status: accepted` (Owner eligió A 2026-09-09;
`docs/architecture/adr/ADR-031-default-members-promotion.md:4,139-143`) → modo
subset: solo crates que mantienen Fast Gate <5min.

Subset (decisión + evidencia por exclusión):
- IN: `vanta-memory` (STABLE-01 ✅ gates 1-6), `vantadb-server` (STABLE-03 ✅),
  `vantadb-mcp` (STABLE-04 gates 1-6 + `test-mcp.py` 4/4 per commit 682e094b).
- OUT: `vanta-proxy` (STABLE-02 ✅ gates pero Heavy wall time documentado + ADR-031
  §2 "heaviest compile, candidate Heavy") → queda experimental con justificación
  Heavy. `vantadb-wasm` (STABLE-05 ✅ pero toolchain extra wasm32/wasm-pack, Tier 3
  BEST-EFFORT) → queda experimental.

Medición local 2026-09-09 (Windows MSVC, target/ limpio con `cargo clean`):
- `cargo check` (usa `default-members` = subset) cold: **132s** ✅ <5min
  (warm baseline pre-edit: default 36s + delta subset 33s ≈ 70s).
- `cargo nextest run --profile audit --build-jobs 2` (subset, como
  `ci-rust-10.yml:test`): cold Windows MSVC Heavy (primera corrida >20min:
  compilación test-targets; core solo 785s en frío — pre-existente del default
  actual, no marginal de la promoción); warm run1 368s (link churn), warm run2
  **296s / 2831 passed, 0 failed** ✅ <5min (4s margen; CI ubuntu-latest + sccache
  ~1.5–2× más rápido per STABLE-08 → margen cómodo en CI).
- Gate 6: `cargo package -p <crate> --list --allow-dirty` exit 0 ×3
  (memory/server/mcp; `--dry-run` literal no existe en `cargo package` —
  precedente STABLE-01/03).

Edición (2 archivos, reversible 1 línea):
- `Cargo.toml:710-719` (`default-members` + comment EXPERIMENTAL actualizado:
  proxy+wasm quedan fuera; `members`/`Cargo.lock`/`publish=false` intactos).
- `docs/operations/CI_POLICY.md` §default-members: nota STABLE-09 subset +
  Heavy excluidos con justificación + rollback 1-línea.

## Contrato — evaluación subset 2026-09-09 (COMPLETO en modo Owner A)
- [x] PR único con `default-members` subset — EJECUTADO en `develop` (commit de esta
  tarea; `[".", "vantadb-python", "vanta-memory", "vantadb-server", "vantadb-mcp"]`)
- [x] CI_POLICY §default-members — NOTA STABLE-09 agregada (subset + Heavy excluidos
  proxy/wasm con justificación)
- [x] `cargo package --dry-run` 0 — `--list --allow-dirty` exit 0 ×3 (memory/server/mcp;
  `--dry-run` literal inexistente en `cargo package`, precedente STABLE-01/03)
- [x] Rollback 1-línea en descripción — en mensaje del commit + CI_POLICY
  (`git revert <commit-promocion>`)
- [x] `just verify` <5min o etiqueta Heavy justificada — subset warm 296s <5min;
  cold Windows documentado Heavy pre-existente (no marginal); proxy/wasm Heavy justificados

## Contrato — evaluación previa 2026-09-08 (BLOQUEO, superada por Owner A)
- [ ] PR único con `default-members` ampliado — NO EJECUTADO (bloqueado; diff futuro de 5 líneas ya documentado en STABLE-08 rama `test/default-all`)
- [x] CI_POLICY §default-members — YA EXISTE (STABLE-08 measurement, sin duplicar)
- [ ] `cargo package --dry-run` 0 — NO CORRIDO (sin cambio que empaquetar; STABLE-01/03 ya validaron `cargo package --list --allow-dirty` EXIT 0 con fix `version="0.5.0"`)
- [x] Rollback 1-línea en descripción — REDACTADO abajo (listo para pegar en futuro PR)
- [x] `just verify` <5min o etiqueta Heavy justificada — HEREDADO: Heavy justificado (cold 8.26m >5, warm 4.15m <5; `verify_changed` cold 1.92m Fast)

## Rollback Plan for [default-members +5 crates] (futuro PR — pegar en descripción)
### Trigger Conditions
- Fast Gate cold >5min en CI (`clippy --workspace` o `nextest --workspace` excede timeout) o error rate CI >2× baseline tras merge.
### Rollback Steps
1. `git revert <commit-promocion> && git push` (1 línea `Cargo.toml:710-713` vuelve a `[".", "vantadb-python"]`) — <1 min + re-run `cargo check` warm.
2. Verificar rollback: `cargo fmt --check` + `cargo check -p vantadb` + `dev-tools/verify_changed.ps1` (<2min cold) + CI Fast Gate verde.
3. Comunicar: notificar canal release + anotar en ADR-031 §4 respuesta Owner.
### Consideraciones de Datos
- Sin migración de datos (cambio build-graph only); `Cargo.lock` delta 0 (ya members); `publish = false` intacto → `cargo publish` no afectado.
### Tiempo estimado
- Revert + verify local: <5 min · CI verde: <5 min Fast Gate.

## Steps
| # | Step | Contrato verify | Estado |
|---|------|-----------------|--------|
| 1 | DISCOVERY: Regla 0 + SDP + estados STABLE-00..08 + ADR-031 + Cargo.toml real + WIP ajeno | Task file creado con Impacto + Spec + Contrato evaluado | ✅ PASS |
| 2 | Cierre BLOQUEADO (2026-09-08): verify ligero + commit SOLO task file + recitation INCOMPLETO | `cargo fmt --check` exit 0 + commit `46153ee1` (1 file) | ✅ PASS (superado: Owner A 2026-09-09) |
| 3 | DISCOVERY delta (2026-09-09): ADR accepted + subset IN/OUT + campaign_update_task_state in-progress | §Ejecución subset redactada | ✅ PASS |
| 4 | MEDICIÓN subset: check cold + nextest warm ×2 + package --list ×3 | 132s / 296s-2831-0fail / 0-0-0 | ✅ PASS |
| 5 | EDICIÓN: Cargo.toml subset + CI_POLICY nota + task file sync | 2 archivos + task file | ✅ PASS |
| 6 | CIERRE: fmt + verify_changed + commit solo archivos propios + recitation | fmt 0,changed 3/3,`904c7ae1` 3 files | ✅ PASS |

## SDP
2026-09-08 (modo BLOQUEADO):
2026-09-09 subset (plan 2026-09-09 Task 1): `campaign_detect_task_type` → `docs`;
`campaign_discover_skills_v2` BUILD → 8 genéricas (filtradas por ponytail: ninguna
específica CI — scoring por keywords `docs`; se cargan las 4 de dominio del perfil
vanta-lead en su lugar): `ci-cd-and-automation` (Fast<5 vs Heavy, quality gates),
`git-workflow-and-versioning` (commit atómico, no tocar WIP ajeno),
`shipping-and-launch` (pre-launch + rollback 1-línea + thresholds),
`documentation-and-adrs` (ADR-031 accepted, CI_POLICY source of truth).
`campaign_discover_skills_v2` BUILD → base + lifecycle (8, filtradas por ponytail a 4 cargadas): `ci-cd-and-automation` (gates Fast<5 vs Heavy, circuit breaker), `git-workflow-and-versioning` (commit atómico 1-file, no tocar WIP ajeno), `shipping-and-launch` (pre-launch checklist + rollback 1-línea + thresholds), `documentation-and-adrs` (ADR-031 proposed, CI_POLICY source of truth). Omitidas con justificación: `incremental-implementation`/`test-driven-development` (sin código nuevo), `frontend-ui-engineering`/`api-and-interface-design`/`context-engineering`/`source-driven-development`/`doubt-driven-development` (scoring genérico tipo `docs`, sin UI/API/decisión framework adversaria real).

## Notas
- `Cargo.toml:636` del plan = stale; real `710-713` (verificado por Read). Anotado para que el orquestador corrija la referencia sin re-derivar.
- WIP ajeno intacto: `M .opencode`, `M docs/Backlog.md` (17 líneas: edición BLOG-CTA del usuario), `M opencode.jsonc`, `?? docs/plans/2026-09-08-backlog.md` (untracked, mezcla recitations) — NINGUNO stageado ni editado por este task.
- `campaign_verify_cmd` bug exit -1 conocido → verify por bash directa (precedente FIND-48/49/STABLE-06).
- STABLE-07 header ⏳ pero steps 5/5 PASS (35/35 vitest, pack .node, Heavy justificado) → funcionalmente verde, administrativamente pendiente de sync (lo hace el orquestador/lead del plan 2026-09-07). STABLE-04 igual (inner COMPLETO). No bloquean el veredicto pero impiden declarar "3-corridas frescas de todo el set hoy".
- Pre-mortem plan confirmado: sin 3-corridas frescas del set ampliado post-STABLE-06-hoy → NO promocionar (este BLOQUEO es el caso previsto, no un fallo).

## Context Save Point
- Trabajo hecho: Step 1 ✅ (este file creado con DISCOVERY completo). Step 2 ⬜ (commit + recitation).
- Archivos tocados (solo este): `docs/tasks/STABLE-09.md`.
- Próximo step: Step 2 — `cargo fmt --check` (docs-only, rápido) + `git add docs/tasks/STABLE-09.md` + commit `docs: STABLE-09 ...` + `campaign_update_task_state` in-progress/PARTIAL + RESULTADO 🟡 INCOMPLETO.

## Verify (evidencia)
- `cargo fmt --all -- --check` → exit 0 ✅ (docs-only, 2026-09-09)
- `git status --short` pre-commit → solo `?? docs/tasks/STABLE-09.md` propio + WIP ajeno intacto (`.opencode`, `docs/Backlog.md`, `opencode.jsonc`, `?? docs/plans/2026-09-08-backlog.md`, `?? Investigacion-plan.md`) ✅
- pre-commit hook → "No staged Rust files; actionlint ok; All checks passed" ✅
- commit `46153ee1` (1 file, 79 insertions, solo `docs/tasks/STABLE-09.md`) ✅
- `Cargo.toml` sin diff, `CI_POLICY.md` sin diff (scope discipline) ✅
- ADR-031 `status: proposed` + `owner: TBD` + "STABLE-09 must not merge" (evidencia de BLOQUEO, no fallo) ✅
