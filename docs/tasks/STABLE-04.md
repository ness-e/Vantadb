# STABLE-04 — Validar vantadb-mcp (gates 1-6 + test-mcp.py)

## Metadata
- **Plan file:** `docs/plans/2026-09-04-durability-release-readiness.md` (Task 7, Wave 2)
- **Campaign ID:** a6f16be4-a2a2-44eb-bfdb-1a84a4b573cf
- **Creado:** 2026-09-05 (DISCOVERY pipeline-full)
- **Estado:** ⏳ IN PROGRESS (DISCOVERY ✅, gates en ejecución)
- **Ruta:** vanta-worker
- **Branch:** develop
- **Commit:** `ci(mcp): validación gates 1-6 (STABLE-04)` — solo si hay edits; si es solo-validación sin cambios, cierre = docs/avance + plan sync, sin commit vacío
- **Tipo:** validate / promotion-gate — verification-only, no nueva pub API
- **Appetite:** max 1d

> SDP: campaign-executor + progreso + ponytail (base) + source-driven-development + security-and-hardening (base type mcp) + incremental-implementation + test-driven-development + context-engineering (lifecycle BUILD). Keywords: mcp/validate/gates/nextest/clippy/deny/promotion. Manifest grep sin candidatos aplicables (icon/slack/brand/academy/claude-api/internal-comms — ninguno del dominio). Total 8.

## DISCOVERY — re-escala de claims con evidencia (sin inflar)

| Claim (Backlog P47 / plan) | Realidad en disco | Veredicto |
|---|---|---|
| `vantadb-mcp/tests/` 11 archivos | `Get-ChildItem` → 11 entries (code/context/mcp_fallback_proxy/mcp_tests/scene/skills/test_embed_texts/thread/wiki_async_ingest/wiki_roundtrip_e2e/wiki_tests) | ✅ claim vigente |
| 72 `mcp_tests` | `rg -c "#\[test\]|#\[tokio::test"`: **91** en `mcp_tests.rs`; **150** total 11 files (7+10+7+9+3+7+5+3+7+1+91); `rg -c "fn .*test"` en mcp_tests.rs = 82 fns; `#[ignore]` = 0 | ⚠️ claim STALE (subcuenta): re-escalado a **91 attrs / 82 fns en mcp_tests.rs, 150 attrs total**. Número de pass real lo da nextest/cargo test, no el conteo estático |
| `test-mcp.py` 37 checks | `skills/vantadb-mcp/scripts/test-mcp.py` (único en repo por glob) → **4 checks** (`initialize`, `tools/list`, `resources/list`, `prompts/list`), exit 0 si 4/4 | ⚠️ claim STALE: re-escalado a **4/4**. "37" no existe en disco |
| protocolVersion 2024-11-05 | `handlers/initialize.rs:7` `LATEST_PROTOCOL_VERSION = "2025-06-18"`, `SUPPORTED = ["2025-06-18", "2024-11-05"]`; `mcp_tests.rs:24-67` fija latest 2025-06-18 + backward-compat echo 2024-11-05; `test-mcp.py:123` handshake con 2025-06-18 | ⚠️ claim STALE parcial: vigente = **latest 2025-06-18, 2024-11-05 solo backward-compat** |
| `serve_lines` con `OpGate` | `server.rs:376,399` `serve_lines` existe (+ `serve_lines_proxy:723`); `rg OpGate vantadb-mcp/` = **0 hits** (OpGate vive solo en vantadb-python/wasm/node + docs) | ⚠️ claim ERRÓNEO: `serve_lines` ✅ existe, `OpGate` ❌ no aplica a este crate (concurrencia vía `Semaphore max_concurrency` + `JoinSet` + `ActiveRequestGuard` en metrics.rs) |
| Skill 79 tools | `skills/vantadb-mcp/SKILL.md:8` → **79 tools** (49 core + 6 skill_* + 8 code_* + 6 wiki_* + 1 context_assemble + 3 scene_* + 6 thread_*) | ✅ vigente (contraste test-mcp.py: solo handshake 4 métodos, no cubre tools) |
| Gates 1-6 P47 | Backlog:622-628: 1=check+fmt+clippy 0 warnings; 2=nextest 0 failed sin `#[ignore]`; 3=deny 0; 4=docs-coverage 0 gaps; 5=workflow CI paths/timeout<5min; 6=`publish=false` ok, `cargo package` sin fallo metadata | Contrato operativo abajo (precedente STABLE-03: gate 6 = `cargo package --list --allow-dirty`) |

**Hallazgo gate 2 (nextest):** `.config/nextest.toml` default-filter excluye `package(vantadb-mcp) and binary(mcp_tests)` → `cargo nextest run -p vantadb-mcp --profile audit` corre solo los otros 10 binaries. `mcp_tests` corre explícito vía `cargo nextest run --package vantadb-mcp --test mcp_tests` (precedente `heavy-certification-50.yml:273-274`) o `cargo test -p vantadb-mcp --test mcp_tests`. Ambos números se reportan.

## Blast Radius
| Dirección | Módulos |
|---|---|
| Callers | `Cargo.toml` workspace members; `vantadb-server` depende de `vantadb-mcp` (path); `vanta-cli server --mcp` spawnea el server; `docs/operations/CI_POLICY.md` experimental-check (`cargo check -p vantadb-server -p vantadb-mcp -p vantadb-wasm`); `heavy-certification-50.yml` corre `mcp_tests` explícito |
| Callees | `vantadb-mcp/src/` 16 entries (lib/server/config/protocol/handlers/{initialize,tools,prompts,resources}/validation/proxy/skills/wiki/context/code/scenes/threads/metrics/axioms/error) + `vantadb` (features cli+server) + `vanta-memory` (fachada ingest wiki) + tokio/axum/reqwest |
| Implicaciones | Solo validación + fix metadata `Cargo.toml` si gate 6 falla (reversible). No toca `src/wal.rs`, `src/vector/`, `src/storage/`. No publica (`publish=false` intacto). Security: superficie LLM-facing (threat model en SKILL.md § Threat Model LLM06) — pero validate-only no cambia código, FASE SECURITY = checklist de lectura, sin cambios de deps → sin `cargo audit` extra |

## Impacto mapeado (Regla 0)
- **Archivos leídos completos (antes de cualquier edit):** `vantadb-mcp/Cargo.toml` (32L — publish=false, deps vantadb path+cli+server, vanta-memory path, tokio/serde/axum; features embed-local/remote-inference), `vantadb-mcp/tests/` (11 files, tamaños listados), `skills/vantadb-mcp/scripts/test-mcp.py` (171L — 4 checks), `skills/vantadb-mcp/SKILL.md` (599L — 79 tools), `docs/Backlog.md:618-641` (contrato P47 10 checks + fila STABLE-04), `.config/nextest.toml` (default-filter excluye mcp_tests), `docs/plans/2026-09-04-durability-release-readiness.md` Task 7, `.opencode/skills/campaign-executor/tasks/STABLE-03.md` (precedente gates), `vantadb-mcp/src/server.rs` (vía codegraph: serve_lines/proxy/config/metrics), `vantadb-mcp/src/handlers/initialize.rs` (vía grep: protocolo)
- **Referencias hacia dentro:** `vantadb-mcp` → `vantadb` (cli+server) + `vanta-memory` + tokio runtime; `tools.rs` 154KB (79 tools); `server.rs` serve_lines/serve_lines_proxy
- **Referencias entrantes:** `vantadb-server` (path dep) + `vanta-cli server --mcp` + CI experimental-check + heavy-cert mcp_tests + skill test-mcp.py
- **Veredicto impacto:** validación pura salvo gate 6 metadata. `Cargo.toml` ya usa solo path deps sin `version=` — gate 6 puede PRE-fallar como en STABLE-01/03 (`dependency vantadb does not specify a version`); fix previsto idéntico (`version="0.5.0"` en path deps, `version.workspace=true` descartado por invalid type map). Si gate 6 pasa sin fix → cero edits → sin commit.

## Spec
| Decisión | Elección | Alternativa descartada | Justificación (evidencia) |
|---|---|---|---|
| Tipo de tarea | Validate-only (sin nueva API) | Feature-add con spec formal | No se añaden `pub fn`/tool/endpoint — solo verificación + fix metadata si gate 6 lo exige. Gate D no dispara (blast radius: 0 archivos editados salvo Cargo.toml metadata; hot path no tocado). |
| Gate 2 nextest | audit-profile (10 binaries) + run explícito mcp_tests + `cargo test` full si nextest filtra | Solo audit-profile | nextest.toml excluye mcp_tests del default-filter → audit solo no cubre el contrato "72 mcp_tests". Precedente STABLE-03 (42 vs 5) idéntico. |
| Gate 6 cargo package | `cargo package -p vantadb-mcp --list --allow-dirty` (metadata check per ADR-031) | `cargo publish --dry-run` / full package | `publish=false` → publish falla esperado; full package falla por dep publish=false no en crates.io (precedente STABLE-03). `--list` ejerce el path de validación metadata. `cargo package` no acepta `--dry-run`. |
| test-mcp.py | 4/4 como pass (no 37) | Inflar a 37 | Disco manda: script tiene 4 checks. Se reporta 4/4 + nota stale. Requiere binario (`vanta-cli`/`vantadb-server` en target/debug o PATH). |
| OpGate | No verificar (no existe en crate) | Añadir OpGate | Fuera de blast radius y de scope; el claim es erróneo, se documenta, no se "arregla" código por un claim stale. |

## Contrato
```
cargo check -p vantadb-mcp --all-targets --all-features ✅ (0 errors/warnings, wall time)
cargo fmt --check ✅
cargo clippy -p vantadb-mcp --all-targets --all-features -- -D warnings ✅ (0 warnings)
cargo nextest run -p vantadb-mcp --profile audit -j 2 ✅ (0 failed; N tests audit-filtered)
cargo nextest run --package vantadb-mcp --test mcp_tests ✅ (0 failed; N tests) + cargo test -p vantadb-mcp --test mcp_tests si aplica
cargo deny check ✅ (0)
pwsh scripts/validate-docs-coverage.ps1 (-ReportOnly) ✅ (0 gaps; MCP N items)
cargo package -p vantadb-mcp --list --allow-dirty ✅ (metadata, publish=false intacto)
python skills/vantadb-mcp/scripts/test-mcp.py ✅ (4/4)
Gate 5 CI: ci-rust-10.yml experimental-check cubre mcp + heavy-cert corre mcp_tests + sin continue-on-error en paths tocados ✅ (lectura)
```

## Steps
### Step 1: Gates 1-3 — check + fmt + clippy + nextest (mcp) ✅/⬜
- **Archivos:** `vantadb-mcp/Cargo.toml`, `vantadb-mcp/src/**`, `vantadb-mcp/tests/**` (lectura; edit solo si gate exige)
- **Acción:** `campaign_verify_cmd` por gate: check, fmt, clippy, nextest audit + nextest mcp_tests explícito. Registrar wall time + conteos. `#[ignore]`=0 ya verificado por rg. Si falla → systematic-debugging, no retry ciego; si causa pre-existente ajena → documentar y cerrar parcial (sin fixes fuera de blast radius sin approval).
- **Verify:** EXIT 0 todos + números reportados.
- **Estado:** ⬜ PENDING

### Step 2: Gates 4-6 + test-mcp.py + Gate 5 CI + cierre ✅/⬜
- **Archivos:** `deny.toml`, `scripts/validate-docs-coverage.ps1` (lectura), `vantadb-mcp/Cargo.toml` (edit solo si gate 6 PRE-fail), `.github/workflows/ci-rust-10.yml` + `heavy-certification-50.yml` (lectura gate 5)
- **Acción:** deny, docs-coverage, `cargo package --list --allow-dirty` (fix `version="0.5.0"` solo si PRE-fail idéntico a STABLE-01/03), build binario si falta + `test-mcp.py` 4/4, lectura gate 5 CI. Luego: verify full (fmt+check+clippy re-run si hubo edit), commit `ci(mcp): validación gates 1-6 (STABLE-04)` SOLO si hubo edits (nunca stagear ajenos — worktree tiene `M .opencode` pre-existente de otras sesiones), task file a COMPLETED, plan Task 7 sync (edición sin stagear per precedente), `skill progreso`, recitation + RESULTADO.
- **Verify:** gates 4-6 ✅ + test-mcp.py 4/4 ✅ + cierre completo.
- **Estado:** ⬜ PENDING

## Notas
- Ponytail: sin bench canonical_p99 (no hot path); solo wall time gates. Sin nuevas deps. `ponytail:` N/A salvo que gate exija workaround con techo.
- campaign_update_task_state in-progress BLOQUEADO por server (FIND-62 + STABLE-06 aún in-progress en campaign-server aunque sus commits 19a9651c/7ff70b01 + plan COMPLETO existen — stale ajeno, no tocar). Se continúa sin claim; el cierre reporta BLOQUEO=state-server si persiste.
- Confusión resuelta sin question (evidencia en disco concluyente en los 5 claims): re-escala documentada arriba, sin inflar.
