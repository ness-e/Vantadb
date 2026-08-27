# STABLE-00 — Checklist y ADR de promoción a default-members (gate 100% estable)

## Metadata
- **Plan file:** docs/plans/2026-08-27-backlog-pipeline.md
- **Creado:** 2026-08-27
- **last-synced:** 2026-08-27
- **Estado:** ✅ COMPLETED
- **Fuente:** docs/Backlog.md P47 — Cargo.toml:636 default-members circuit breaker
- **Esfuerzo:** 🟡 4h
- **Prioridad:** 🔴 Alta
- **Tipo:** docs / ADR — governance
- **Turns estimados:** 3

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `Cargo.toml:636` (workspace default-members), `docs/operations/CI_POLICY.md` (§Experimental Circuit Breaker), `dev-tools/verify.ps1` (gate local) leen o documentan la lista |
| Callees | Ninguno — docs-only, no compila código; `cargo check`/`clippy`/`nextest`/`deny` son gates externos verificados por CI |
| Implicaciones | Sin cambio de comportamiento runtime. Solo criterio de promoción + doc. No toca `src/`, `vanta-*/src/`, ni publica crates (`publish=false` intacto). Reversible en 1 línea. |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:**
  - `Cargo.toml:620-656` (36 líneas — `[workspace]` members 6 + default-members 2 + exclude + comentario CATEGORY: EXPERIMENTAL)
  - `docs/operations/CI_POLICY.md` (302 líneas — §Experimental Crate Circuit Breaker líneas 109-143 + Fast Gate §1 + inventory 14 workflows)
  - `dev-tools/verify.ps1` (92 líneas — gate local fmt/check/clippy/audit/deny/nextest/coverage/docs-coverage, con -E RESOURCE-GUARD filters)
  - `dev-tools/gate-common.ps1` (22 líneas — Get-CoreFeatures cli,fjall,memmap2,fs2,roaring)
  - `scripts/validate-docs-coverage.ps1` (197 líneas — 6 checks: SDK/config/error/CLI/python/MCP)
  - `docs/architecture/adr/` (16 ADRs existentes: 001,014-030; template `docs/_templates/adr.md`)
  - `.github/workflows/ci-rust-10.yml` (jobs fmt/clippy/semver/adr-gate/test/windows/macos/msrv/minimal/coverage/wasm-test/experimental-check/audit/miri)
  - `.github/workflows/release-npm-61.yml` (timeout 10, tests 26s Fast Gate) y `release-npm-node.yml` (matrix 7 targets, 1 continue-on-error flag)
  - `vanta-memory/Cargo.toml`, `vanta-proxy/Cargo.toml`, `vantadb-wasm/Cargo.toml`, `vantadb-ts/package.json`, `vantadb-node/package.json`
  - `docs/Backlog.md:721-750` (P47 contrato 10 checks + 9 filas STABLE)

- **Referenciados hacia dentro (imports/includes):**
  - `Cargo.toml` lista en `[workspace].members` (6) y `[workspace].default-members` (2); CI y verify.ps1 leen esa lista implícitamente via `cargo check -p vantadb` sin `--workspace`.
  - `CI_POLICY.md` §Experimental Crate Circuit Breaker referencia `Cargo.toml` default-members + 5 reglas de exclusión (`--exclude` en clippy/coverage, `experimental-check` job).
  - `verify.ps1` referencia `gate-common.ps1:Get-CoreFeatures` y `scripts/validate-docs-coverage.ps1`.

- **Referencias entrantes (a los editados):**
  - `Cargo.toml` default-members es leído por `cargo` (workspace resolver), por `ci-rust-10.yml:experimental-check` (cargo check -p server/mcp/wasm) y por `verify.ps1` (cargo check -p vantadb con features core).
  - `CI_POLICY.md` es referencia normativa para `release-ci.md` y `.opencode/AGENTS.md` Regla 2 pipeline.
  - ADRs son referenciados por `adr-gate` job (fail si cambio API sin ADR) y por `docs/operations/CI_POLICY.md`.

- **Veredicto impacto:** mínimo y reversible — 2 archivos de documentación tocados (nuevo ADR + 1 sección en CI_POLICY), 0 crates recompilados, 0 APIs públicas cambiadas, 0 toolchain extra instalado en este step. El cambio es 100% doc-driven y deferre la edición de `Cargo.toml` a STABLE-09 tras validación.

## Spec

| Decisión | Elección | Evidencia |
|----------|----------|-----------|
| Número de ADR | `ADR-031-default-members-promotion.md` (siguiente libre tras 030) | `docs/architecture/adr/` lista ADR-030 como último; secuencia incremental |
| Formato ADR | Frontmatter `title/status/tags/created/last_reviewed/owner` + §Context/§Decision/§Consequences/§Alternatives/§References siguiendo `docs/_templates/adr.md` y ADR-027/028 como ejemplo | Template 24 líneas + ADRs reales 80-116 líneas |
| Tabla 10 checks | Reproducir exactamente contrato P47 (Backlog:721-736) numerados 1..10 con comando y criterio de pass | Backlog:726-736 define 10 gates con 3 corridas, cargo clean/npm ci |
| Coste por crate | Tabla con 7 columnas: crate, `cargo check` (s), `clippy` (s), `nextest`/`vitest` (s), toolchain extra, `Cargo.lock` delta (KB), nota Fast/Heavy | Medición local 2026-08-27: `cargo check -p` ~3-32s (wasm 3.5s, mcp 6.6s, server ~21s, proxy 32s, memory 36s); sin toolchain extra salvo wasm (`wasm32`+`wasm-pack`) y node (`napi`); Cargo.lock no crece (crates ya en workspace) |
| Gate reversible | Sección §Reversibilidad: `git revert` de 1 línea en `Cargo.toml:636` + revert CI_POLICY §default-members; `publish=false` intacto, no afecta `cargo publish` | P47 origen: "deja la promoción como cambio reversible en 1 línea" + STABLE-09 contrato |
| CI_POLICY update | Añadir §Promoción a default-members (o sub-sección bajo Circuit Breaker) que enlaza a ADR-031 y lista los 10 checks como DoD | Contrato: "docs/operations/CI_POLICY.md §default-members menciona ADR" — el § actual es 109-143 sin referencia a ADR futuro, debe citar ADR-031 |
| Question gate | Sección §Pregunta al owner en ADR con 2 opciones: <5 min Fast Gate (requisito duro) vs re-etiquetar como Heavy con justificación (STABLE-08) + registro de respuesta pendiente | STABLE-00 contrato: "Gate: `question` al owner para aprobar el umbral Fast Gate (<5 min vs Heavy) antes de tocar Cargo.toml" — ADR debe dejar DRAFT hasta respuesta |
| Idioma | Inglés (source of truth para docs/architecture y docs/operations) | Doc Language Split: English para architecture/ops, Spanish solo Backlog/avance |
| No tocar Cargo.toml | En este STABLE-00 no se edita `Cargo.toml:636` — promoción diferida a STABLE-09 | Contrato STABLE-00: "antes de tocar Cargo.toml" — este task solo escribe criterios |

## Contrato

```
existe docs/architecture/adr/ADR-031-default-members-promotion.md con:
  - tabla 10 checks (cargo check/clippy/nextest/deny/docs-coverage/workflow timeout/wasm-pack/npm pack/verify <5min/ADR reversible) + nombres de comando exactos
  - tabla de coste por crate (tiempo cargo check/clippy/nextest, toolchain extra, tamaño Cargo.lock)
  - question al owner sobre umbral Fast Gate (<5 min vs Heavy) registrada en ADR (§Pregunta al owner)
  - §Reversibilidad 1 línea
existe docs/operations/CI_POLICY.md §default-members menciona ADR-031 (grep -n "ADR-031" CI_POLICY.md -> hit)
cargo fmt --check == 0 (docs-only, no formato roto)
cargo clippy -p vantadb --all-targets no regresión (solo docs)
scripts/validate-docs-coverage.ps1 no aplica gaps nuevos (docs-only)
```

## Herramientas

- cargo, pwsh, git
- skill documentation-and-adrs (plantilla ADR)
- skill ponytail (ladder: doc-driven, mínimo código)
- skill progreso (cierre)

## Skills

- **campaign-executor** (base, pipeline-full)
- **progreso** (base, cierre)
- **ponytail** (base, lazy full)
- **documentation-and-adrs** (ADRs, plantillas, 7/10 rating) — necesaria para formato ADR, consecuencias y referencias
- **writing-guidelines** (SDP discovery: voz/tono docs — no aplica styling pesado, pero valida que ADR en inglés y CI_POLICY en inglés cumplen handbook)
- **spec-driven-development** (SDP: Lifecycle DEFINE — este task es spec-first doc-driven; la spec table arriba es la source de verdad antes de promoción)
- **ci-cd-and-automation** (SDP: SHIP — CI_POLICY es pipeline doc; valida que la sección default-members aluda a workflow timeouts <5 min vs Heavy)
- **git-workflow-and-versioning** (SDP: SHIP — ADR reversible en 1 línea, commit atómico, no tocar versión Cargo.toml)

> SDP: 4 discovery skills adicionales sobre base 4 = 8 totales (≤8 límite). Keywords grepped: adr/promotion/default-members/ci-policy → documentation-and-adrs hit; lifecycle DEFINE/PLAN/SHIP → spec-driven + ci-cd + git-workflow; writing-guidelines para voz del ADR.

## Steps

### Step 1: Escribir ADR-031-default-members-promotion.md con 10 checks + coste + question gate
- **Archivos:** `docs/architecture/adr/ADR-031-default-members-promotion.md` (nuevo)
- **Acción:** Crear ADR siguiendo plantilla con Context (cargo 636 deja server/mcp/wasm/memory/proxy fuera, P47 sin criterios), Decision (§Criterios de promoción: tabla 10 checks numerados con comando exacto y criterio pass en 3 corridas, §Tabla de coste por crate con tiempos medidos 2026-08-27, §Reversibilidad, §Pregunta al owner), Consequences (pros/cons), Alternatives, References. Incluir toolchain extra (wasm32, wasm-pack, napi) y Cargo.lock delta = 0 (ya members). Question registrada como pendiente con owner TBD y 2 opciones (<5 vs Heavy).
- **Verify:** `Test-Path docs/architecture/adr/ADR-031-default-members-promotion.md` → true + `Select-String -Pattern "ADR-031" docs/architecture/adr/ADR-031-default-members-promotion.md | Measure` ≥3 hits + `Select-String -Pattern "^\| 1\."` o tabla 10 checks presente + `Select-String -Pattern "Pregunta al owner|Question to owner"` → hit
- **Estado:** ✅ COMPLETED (2026-08-27 — ADR creado 302 líneas, 10 checks tabla + coste per crate + Question to Owner §4 con opciones A/B registrada como pending)

### Step 2: Actualizar docs/operations/CI_POLICY.md §default-members para mencionar ADR-031
- **Archivos:** `docs/operations/CI_POLICY.md` (editar §Experimental Crate Circuit Breaker, línea ~109-143)
- **Acción:** Añadir sub-sección `#### Promoción a default-members — DoD y ADR-031` o inline note en reglas de promoción: "Para promover un crate experimental a estable, deben pasar los 10 checks de ADR-031 en 3 corridas; ver ADR-031 para criterios, coste y reversibilidad; promoción = 1 línea en Cargo.toml:636". Mantener reglas existentes (CATEGORY: EXPERIMENTAL). Añadir link relativo `../architecture/adr/ADR-031-default-members-promotion.md`. No tocar Cargo.toml.
- **Verify:** `Select-String -Path docs/operations/CI_POLICY.md -Pattern "ADR-031"` → hit (líneas 137,139,145,153) + `Select-String -Pattern "default-members"` → hit (6 hits) + `Get-Content CI_POLICY.md | Select-String -Pattern "10 checks|10 gates"` → hit
- **Estado:** ✅ COMPLETED (2026-08-27 — CI_POLICY editado: §Promotion subsección + link relativo + 10-check DoD + rollback 1 línea + Owner gate bloqueado)

### Step 3: Verify mecánico + commit + progreso
- **Archivos:** ambos de S1-S2
- **Acción:** `cargo fmt --check` (docs-only, debe pasar), `pwsh scripts/validate-docs-coverage.ps1 -ReportOnly` (0 gaps nuevos), verificar `git diff --stat` solo toca 2 archivos, commit `docs: STABLE-00 ADR-031 + CI_POLICY default-members DoD`, actualizar plan file y task file a COMPLETED, ejecutar skill progreso (mover backlog a avance si aplica — STABLE-00 vive en P47, no se archiva hasta P47 completo, solo se marca task file).
- **Verify:** `git log --oneline -1` contiene STABLE-00 + `git status --short` clean + `Test-Path ADR-031` true + `Select-String ADR-031 CI_POLICY.md` true
- **Estado:** ✅ COMPLETED (2026-08-27 — cargo fmt --check ✅, clippy -p vantadb ✅, validate-docs-coverage 0 gaps ✅, commit 8e206431)

## Dependencias
- Requiere: ninguno — STABLE-00 es fundación de P47
- Bloquea: STABLE-01..09 (todos esperan criterios y question gate)

## Notas
- Ponytail: no medir bench `canonical_p99` aquí (no hot path); solo wall time de cargo check/clippy/nextest. No añadir crates a Cargo.toml en este step.
- No tocar `dev-tools/verify.ps1` en STABLE-00 (STABLE-08/09 lo tocan si hace falta para wasm target).
- Owner question queda registrada como pending en ADR hasta respuesta explícita; STABLE-01..08 pueden validar gates pero STABLE-09 bloquea hasta GO.

## Context Save Point
- Trabajo previo: S1-S3 ✅ + verify full (fmt, clippy, docs coverage 0 gaps) + commit 8e206431
- Archivos tocados: docs/architecture/adr/ADR-031-default-members-promotion.md, docs/operations/CI_POLICY.md, .opencode/skills/campaign-executor/tasks/STABLE-00.md
- Próximo step: ninguno — tarea cerrada

## Verify (evidencia)
- `Test-Path ADR-031-default-members-promotion.md` → True ✅
- `Select-String ADR-031 CI_POLICY.md` → 4 hits (líneas 137,139,145,153) ✅
- `Select-String "default-members" CI_POLICY.md` → 6 hits ✅
- `Select-String "10 checks" CI_POLICY.md` → hit ✅
- `Select-String "^\| [0-9]" ADR-031` → Count 10 (checks 1-10) ✅
- `Select-String "Question to Owner" ADR-031` → hit (§4) ✅
- `Select-String "Cargo.lock" ADR-031` → hit (cost table, delta 0) ✅
- `cargo fmt --check` → ✅ (exit 0)
- `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` → ✅ (1.9s)
- `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps (7/7 checks) ✅
- `git log --oneline -1` → 8e206431 docs: STABLE-00 ✅
- `git diff --cached --stat` pre-commit → 3 files, 357 insertions ✅
