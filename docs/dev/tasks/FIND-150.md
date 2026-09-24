# FIND-150: Higiene dependencias + DoD v2 — llvm-cov, machete, OSV-Scanner

## Metadata
- **Plan file:** docs/dev/plans/2026-09-24-harness-gaps.md
- **Fuente:** Backlog FIND-150 + plan §FIND-150
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Tipo:** CI/Release (devops)
- **Turns estimados:** 8
- **Creado:** 2026-09-24T12:00
- **last-synced:** 2026-09-24T12:00
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 5 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | CI workflows (ci-rust.yml jobs audit/deny/coverage), ci-gate REQUIRED list, DoD v2 consumers |
| Callees | vantadb-server dev-deps, vantadb-node build-dep, desktop build-dep, osv-scanner upstream, cargo-llvm-cov/machete tooling |
| Implicaciones | Sin cambio de comportamiento público; solo limpieza dev-deps + jobs CI aditivos; sin migración; tests de server recompilan contra menos dev-deps |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `deny.toml` (81L), `Cargo.toml` raíz (785L, incl. `[package.metadata.cargo-machete]` ignored getrandom), `.github/workflows/ci-rust.yml` (620L, jobs coverage 80% + audit + deny existentes), `.github/workflows/ci-gate.yml` (61L), `.opencode/references/definition-of-done.md` (144L, DoD v2 líneas 127-130), `vantadb-server/Cargo.toml`, `vantadb-server/src/main.rs` (comentario clap:34-36), `vantadb-node/Cargo.toml` + `build.rs` (napi_build::setup:2), `desktop/src-tauri/build.rs` (tauri_build::build:10)
- **Archivos referenciados hacia dentro:** server dev-deps → `vantadb-server/tests/*` (solo serde_json usado); `napi-build` → `vantadb-node/build.rs`; `tauri-build` → `desktop/src-tauri/build.rs`
- **Archivos que referencian a los editados:** ci-gate.yml REQUIRED ← nombres de jobs de ci-rust.yml; DoD v2 ← herramientas de CI; release-plz/semver no afectados
- **Veredicto impacto:** BAJO — remoción de dev-deps sin uso verificado por grep (chrono/clap/console/fs2/indicatif/serde en server: 0 usos reales); 2 falsos positivos de machete con uso real en build.rs → justificar vía `ignored`, no remover; jobs CI nuevos en paralelo sin tocar Fast Gate crítico

## Contrato

"coverage gate ≥70% vigente en CI (job coverage 80% existente) + `cargo machete --with-metadata` exit 0 + OSV-Scanner job en CI junto a `cargo audit` + DoD v2 actualizado con las 3 herramientas + CI YAML válido"

## Spec (SDD)

No aplica — sin símbolos públicos nuevos (solo remoción dev-deps, jobs CI, docs). Phase 1b: ninguna señal feature-add.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) Fast Gate <5min: jobs nuevos en paralelo, sin `continue-on-error` nuevo sin CATEGORY; (2) Regla 6 saldo neto ≤0: solo remover, no agregar deps runtime; (3) no tocar `.opencode/agents/*` ni `evals/` (FIND-148/149/151 en paralelo); (4) licencias siguen en deny.toml (OSV no duplica licencias)
- **Comandos de verificación:** `cargo machete --with-metadata . vantadb-python vantadb-server vantadb-mcp vantadb-wasm vanta-memory vanta-proxy` (exit 0); `cargo check -p vantadb-server --all-targets`; `cargo fmt --check`; YAML read-back
- **Deuda pendiente:** baseline llvm-cov local completo no corrido en Windows (coste); enforcement vía CI job coverage 80% existente; udeps nightly queda informativo (pre-mortem plan)

## Recitation (canónico)

| Campo recitation (MCP) | Fuente |
|------------------------|--------|
| `activeGoal` | FIND-150: higiene dependencias + DoD v2 |
| `lastAction` | Discovery inline completo + machete baseline (ver Notas) |
| `result` | PARTIAL (steps ejecución pendientes) |
| `nextAction` | Step 1: remover 6 dev-deps de vantadb-server/Cargo.toml |
| `contract` | ver ## Contrato + Invariantes |
| `nextTask` | FIND-149 (Wave 1) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto ≤0:** se eliminan 6 dev-deps sin uso (pago de deuda P2-5/P2-8) + 0 deps nuevas (OSV-Scanner es action CI, no crate). Sin deuda nueva.

## Definition of Done (3 niveles)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable cumple + fmt/clippy/check scoped + machete exit 0 |
| **Commit** | Commits atómicos por repo (VantaDB + .opencode), conventional + FIND-150, `git add` scoped |
| **Release** | N/A (tarea tooling/CI, sin cambio user-visible; changelog no aplica) — justificar en Notas |

## Herramientas necesarias

- cargo-machete 0.9.2, cargo-llvm-cov 0.8.7 (instalados, verificados)
- osv-scanner (solo CI via action; sin binario local)

**Skills cargadas (SDP):** ci-cd-and-automation (wiring jobs CI + gates), doubt-driven-development (verificación adversarial hallazgos machete) + base campaign-executor/progreso/ponytail(lite).
SDP nota: sin acceso MCP campaign_* en este contexto → discovery manual equivalente (blast radius + grep + verify mecánico).

## Investigation Notes

- `cargo machete` raíz (crate `.`): limpio. Multi-path: ver corrección doubt-driven abajo.
- **Corrección doubt-driven (reconcile Step 1):** el primer intento removió 6 dev-deps
  del server y `cargo check --all-targets` falló con 17 errores (E0432/E0433/E0599/E0277).
  Causa raíz: `vantadb-server/tests/*.rs` incluyen el harness compartido vía
  `#[path = "../../tests/common/mod.rs"]` (mcp_integration.rs:9-10, server.rs:9), y
  root `tests/common/mod.rs` usa `fs2` (:7), `serde` (:10), `console` (:6),
  `indicatif` (:9), `chrono` (:299,351,394-395,540) — fuera del alcance del grep
  inicial y del heurístico per-dir de machete. Decisión final: remover SOLO `clap`
  (0 usos, solo comentario main.rs:34-36) + justificar 5 vía `ignored` + 2 build-deps
  (`napi-build` usado en vantadb-node/build.rs:2, `tauri-build` en
  desktop/src-tauri/build.rs:10). `cargo check -p vantadb-server --all-targets` ✅
  tras la corrección; `cargo machete --with-metadata` (9 paths) exit 0 ✅.
- OSV-Scanner local v2.6.0 (`go install .../v2/cmd/osv-scanner@v2.6.0`, Go 1.26.2):
  primer scan exit 1 con 2 hallazgos reales (lru 0.16.4 RUSTSEC-2026-0253, paste
  1.0.15 RUSTSEC-2024-0436) + 1 ignore sin uso (0429, solo en desktop lock).
  Resolución: espejar triage de audit.toml/deny.toml en osv-scanner.toml (4 stanzas
  con owner+expiry), eliminar 0429 del root (config no propaga). Re-scan: exit 0,
  "No issues found", 4 vulns filtrados con razón ✅.
- CI existente: job `coverage` (llvm-cov + threshold 80% ≥ contrato 70%), jobs
  `audit` + `deny` — FIND-150 agrega jobs `osv` + `machete` + REQUIRED en ci-gate.

### Web research digest (Step 5 — ≤500 palabras, URLs verificadas vía webfetch)

- **OSV-Scanner landing** (https://google.github.io/osv-scanner/ ✅): frontend oficial
  de OSV.dev; advisories de fuentes abiertas autoritativas (incl. RustSec) con mapeo
  preciso de versiones → menos notificaciones accionables. Uso: CLI o lib Go.
- **GitHub Action** (https://google.github.io/osv-scanner/github-action/ ✅):
  dos reusable workflows pinnados (`osv-scanner-reusable.yml@v2.6.0` full scan,
  `-pr.yml@v2.6.0` solo vulns nuevas del PR), inputs `scan-args`, `upload-sarif`,
  `fail-on-vuln` (default true). Decisión: NO usar el reusable (requiere
  `security-events: write` + check-runs con nombres fuera de control → riesgo con
  ci-gate fail-closed); en su lugar job `osv` dentro de ci-rust.yml con binario
  pinnado (`go install ...@v2.6.0`, GOTOOLCHAIN go1.26.2) y nombre determinista.
- **Installation** (https://google.github.io/osv-scanner/installation/ ✅): binarios
  SLSA3 por release + `go install github.com/google/osv-scanner/v2/cmd/osv-scanner`
  (requiere Go 1.26.2+ — runner ubuntu + local cumplen).
- **Configuration** (https://google.github.io/osv-scanner/configuration/ ✅):
  `osv-scanner.toml` junto al lockfile (no propaga a subdirs), `[[IgnoredVulns]]`
  con `id` + `reason` + `ignoreUntil` (formato RFC3339 completo — fecha sola falla
  el parse, verificado empíricamente), `[[PackageOverrides]]` para licencias/paquetes.
- **Scan source** (https://google.github.io/osv-scanner/usage/scan-source ✅):
  `osv-scanner scan source --lockfile <path> --config <toml> <dir>` (subcomando
  default); `-r` recursivo; respeta `.gitignore` salvo `--no-ignore`.
- **Release v2.6.0** (https://github.com/google/osv-scanner/releases/tag/v2.6.0 ✅):
  existe y es la versión pinnada en CI + local (misma versión ambos lados).
- **llvm-cov + nextest**: sin fetch externo — evidencia in-repo: job `coverage` en
  ci-rust.yml:359-376 usa `cargo llvm-cov nextest --profile audit --workspace`
  + threshold 80% vía JSON report (líneas 377-396); binario local 0.8.7 verificado.
  El gate 80% CI subsume el contrato 70% piloto (run local completo omitido por
  budget >10min — documentado en deuda, no como número).
- **cargo-machete**: sin fetch externo — evidencia local: v0.9.2, exit 0 en 9 paths
  con `--with-metadata`; limitación documentada (ciego a `#[path]` cross-dir y a
  build.rs) compensada con `ignored` justificados.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 |
| Pendientes de ejecución | 5 (Steps 1-5; Step 6 cierre) |
| % completado | 30% |

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — toca dependencias (remoción) + CI security (OSV junto a audit): quitar deps reduce superficie; `cargo audit`/`deny` no afectados (sin cambios de lockfile funcional — dev-deps removidas sí tocan lockfile: verificar `cargo check` regenera + audit pasa). OSV no duplica licencias (siguen en deny.toml).
- [x] **PERFORMANCE** — no toca hot paths. Justificado: solo Cargo.toml/tests-deps + YAML + docs.

## Steps

### Step 1: Remover dev-dep sin uso de vantadb-server (solo `clap`)
- **Archivos:** `vantadb-server/Cargo.toml`
- **Acción:** eliminar `clap` de `[dev-dependencies]` (0 usos; resto restaurado tras
  corrección doubt-driven — usados vía harness `#[path]`); comentario + `ignored`
- **Verify:** `cargo check -p vantadb-server --all-targets` ✅ (exit 0)
- **Estado:** ✅ COMPLETED

### Step 2: Justificar falsos positivos machete (napi-build, tauri-build + 5 server)
- **Archivos:** `vantadb-node/Cargo.toml`, `desktop/src-tauri/Cargo.toml`, `vantadb-server/Cargo.toml`
- **Acción:** `ignored` con comentario (build.rs:2/10 + harness `#[path]`)
- **Verify:** `cargo machete --with-metadata` (9 paths) → exit 0 ✅
- **Estado:** ✅ COMPLETED

### Step 3: Cablear OSV-Scanner + machete en CI
- **Archivos:** `.github/workflows/ci-rust.yml`, `.github/workflows/ci-gate.yml`, `osv-scanner.toml` (nuevo)
- **Acción:** jobs `osv` + `machete` en ci-rust.yml; REQUIRED en ci-gate; `osv-scanner.toml` en paths triggers; triage espejado audit/deny
- **Verify:** YAML parse OK + jobs presentes + OSV local exit 0 ✅
- **Estado:** ✅ COMPLETED

### Step 4: Actualizar DoD v2 con las 3 herramientas
- **Archivos:** `.opencode/references/definition-of-done.md:127-130`
- **Acción:** v2 nombra `cargo llvm-cov nextest` (70%, gate CI 80%) + `cargo machete` + `osv-scanner`; eliminado flag inexistente `--coverage`
- **Verify:** read-back ✅
- **Estado:** ✅ COMPLETED

### Step 5: Web research digest (llvm-cov nextest + OSV CI wiring)
- **Archivos:** este task file (Investigation Notes)
- **Acción:** 6 URLs OSV verificadas vía webfetch + evidencia in-repo llvm-cov/machete
- **Verify:** cada URL OSV resolvió 2xx ✅
- **Estado:** ✅ COMPLETED

### Step 6: Verify full + commits + RESULTADO
- **Archivos:** todos los tocados
- **Acción:** fmt/clippy scoped + machete + docs-coverage + OCR review + 2 commits (VantaDB, .opencode) + RESULTADO §7
- **Verify:** comandos del contrato en verde
- **Estado:** ⬜ PENDING

## Dependencias
- Sin bloqueantes (Wave 0). Next: FIND-149 (Wave 1).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** doubt-driven-development (adversarial inline: cada hallazgo machete verificado contra código antes de actuar) + OCR delegation en Step 6. Sin sub-agente distinto disponible en este contexto → fallback doubt-driven mandatorio (§Phase 5).
- **Enfoque:** ¿remover vs justificar correcto? Remover solo con 0 usos verificados; justificar con uso real citado.
- **Cómo se probó:** grep mecánico + cargo check + machete exit 0 (evidencia abajo en Notas al cerrar).
- **Checklist anti-hábitos tóxicos:**
  - [ ] No inventar salidas de comandos no ejecutados.
  - [ ] No saltarse clarificación (contrato del plan es explícito; sin ambigüedad → sin question).
  - [ ] No declarar done sin verify mecánico.
  - [ ] No ignorar fallos parciales.
  - [ ] No dar búsqueda por saturada (multi-path machete + grep por crate).
  - [ ] No copiar sin citar.
  - [ ] No reintentar sin diagnóstico.
  - [ ] Cada paso conectado al contrato.
  - [ ] Sin degradación de checks en paths seguridad.
  - [ ] Budget explícito (STOP si verify local >10min → delegar/documentar).
- **Veredicto:** ⏳ pendiente (al cierre)

## Notas
- `git status`: solo `M docs/dev/plans/2026-09-24-harness-gaps.md` pre-existente (no tocado por esta tarea; es del lead).
- Branch: develop. `.opencode/` es repo separado (su commit va aparte).
- Gate P: no dispara (blast radius ≤6 archivos, sin hot path/API pública, contrato mecánico). Gate D: no dispara (sin símbolos públicos nuevos, no feature-add). Gate V: pendiente (0 fallas mismo-error hasta ahora). Gate C: colaterales ninguno hasta ahora.
