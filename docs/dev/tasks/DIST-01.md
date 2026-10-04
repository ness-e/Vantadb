---
title: "TASK DIST-01: Publicar vanta-memory (quitar publish = false)"
kind: task
description: "Desbloquear la publicación de vanta-memory: dry-run verde + smoke externo + decisión release-plz (hold hasta bootstrap). No publica de verdad (owner/F6)."
---

# TASK DIST-01: Publicar `vanta-memory` (quitar `publish = false`)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 14, F0)
- **Fuente:** DELTA 2026-09-30 (P0) — "vanta-memory es el único path monetizable del tier embebido; invisible = no existe"
- **Esfuerzo:** 🟢 2-4h | **Appetite:** max 1d | **Prioridad:** 🔴
- **Tipo:** Release/Packaging (config + docs; sin lógica de negocio)
- **Creado:** 2026-10-04T10:20Z | **last-synced:** 2026-10-04T11:10Z
- **Estado:** ⏳ IN PROGRESS (steps 1-5 ✅; review P2-01 + commits pendientes)
- **Campaign ID:** master-plan-0.9.0-20261004
- **Incógnitas (uphill):** 0 abiertas (dry-run autoritativo ✅ verde)
- **Pendientes (downhill):** 1 step (cierre: review + commits)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-mcp` (path dep, `llm-driver`), `vanta-proxy` (path dep, `llm-driver`), `desktop/src-tauri` (workspace aislado, path dep), workspace members/default-members + `exclude` del root |
| Callees | `vantadb` (path dep — ahora con `version = "0.8.0"`), crates.io: serde/serde_json/thiserror/toml/tracing/chrono (+reqwest/tiktoken-rs opcionales) |
| Implicaciones | Sin cambio de código ni de runtime. Path deps locales siguen resolviendo al workspace (la `version` agregada matchea la local 0.8.0). Nuevo efecto: el crate pasa a ser empaquetable/publicable y `release-plz` debe quedar explícitamente en hold |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vanta-memory/Cargo.toml` (69L), `release-plz.toml` (64L), `vanta-memory/src/lib.rs` (66L), `Cargo.toml` root (814L, secciones relevantes), `.github/workflows/release.yml` (71L), `vanta-memory/tests/smoke.rs` (21L), `vanta-memory/src/core/mod.rs` (57L), `vanta-memory/src/core/hooks/mod.rs` (14L), `vanta-memory/src/core/hooks/auto_recall.rs:220-269` (RecallConfig)
- **Archivos referenciados hacia dentro (imports/deps):**
  - `vanta-memory/Cargo.toml` ← consumido por `vanta-proxy/Cargo.toml:33`, `vantadb-mcp/Cargo.toml:27`, `desktop/src-tauri/Cargo.toml:46` (path deps — intactos)
  - `vanta-memory` ← workspace members (`Cargo.toml:779`), `default-members` (`Cargo.toml:791`), `exclude` del root package (`Cargo.toml:52`)
  - `release-plz.toml` ← `.github/workflows/release.yml:38` (acción release-plz) + `docs/dev/workflow/PUBLISH.md`
- **Archivos que referencian a los editados (referencias entrantes):** grep `vanta-memory|vanta_memory` en Cargo.tomls → 8 matches (3 consumidores + workspace + self); grep `publish = false` → 11 crates (este sale de la lista; quedan 10 + `fuzz` fuera del workspace)
- **Veredicto impacto:** **BAJO** — cambio de metadata/manifiesto, sin código. Único efecto semántico: `vanta-memory` deja de ser "no publicable" a nivel cargo; se contrarresta con hold explícito en `release-plz.toml` (`release = false`) para que el próximo Release PR (#238) no intente publicarlo sin el bootstrap. Los consumidores in-process no se tocan (path deps siguen funcionando).

## Contrato

> Del plan (Task 14) + prompt de tarea. **No incluye publish real** (owner/F6 al cierre del plan).

1. `cargo publish --dry-run -p vanta-memory` verde (empaqueta + verificación de build aislada contra crates.io `vantadb 0.8.0`) — ⬜ pendiente Step 2
2. Smoke del crate como dependencia externa: proyecto tmp con `vanta-memory` como path dep compila (`cargo check` exit 0) — ⬜ pendiente Step 5
3. Decisión release-plz registrada: **hold explícito** (`[[package]] release = false` + comentario con checklist de unblock) — coordinado con #238 — ⬜ pendiente Step 3
4. Workspace intacto: `cargo metadata` OK; consumidores locales sin cambios — ⬜ pendiente Step 2

## Spec (SDD — decisión de release, no feature-add)

> No agrega símbolos públicos nuevos (Phase 1b no dispara). La sección se llena igual porque hay decisiones técnicas de release.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | release-plz: hold vs publish | A) **hold `release = false`** (seguro; unblock explícito owner) / B) `publish = true` (entra al próximo Release PR pero el publish automático FALLA: no hay trusted publisher y el crate no existe) | A | ✅ decidido-por-evidencia: crates.io docs "initial publish requires an API token" (trusted publishing prereq) + release-plz docs (`release=false` = ignorar paquete; `publish=false` = solo salta cargo publish) + historial del repo (vanta-memory nunca entró a un Release PR con `publish=false`) |
| 2 | Metadata crates.io | A) license+repo+homepage+docs+keywords+categories, **sin README** / B) + README nuevo (scope docs) | A | ✅ decidido-por-evidencia: crates.io exige description+license (Cargo book); README se difiere a DIST-04 (dueño de la superficie documental del crate) |
| 3 | Pin de versión del dep `vantadb` | A) `version = "0.8.0"` junto a `path` (obligatorio para publicar) / B) path-only (dry-run FALLA: "dependencies specified with only a path are not permitted on crates.io") | A | ✅ decidido-por-evidencia: Cargo book §"Local paths in published crates" |
| 4 | Doc-sync `lib.rs` | A) actualizar la nota "Stability" (hoy dice `publish = false`) / B) dejarla stale | A | ✅ decidido-por-evidencia: Regla 3 (doc viva); la nota viaja al crate publicado (docs.rs). `docs/api/VANTA_MEMORY.md` NO se toca (lo cierra DIST-04 por diseño) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) consumidores in-process (`vantadb-mcp`, `vanta-proxy`, `desktop`) siguen compilando sin tocar sus manifests; (2) sin publish real, sin `--token`, sin tags; (3) el próximo Release PR (#238) NO debe intentar publicar `vanta-memory` (hold vigente); (4) `Cargo.lock` sin cambios; (5) WIP ajeno intacto (`opencode.jsonc`, plan file, `src/sdk/**`, `tests/edge_cases.rs`, `benchmarks/`).
- **Comandos de verificación:** `cargo publish --dry-run -p vanta-memory --allow-dirty` (exit 0) · `cargo check` en proyecto tmp (exit 0) · `python -c "import tomllib; tomllib.load(open('release-plz.toml','rb'))"` (exit 0).
- **Deuda pendiente:** README del crate ausente (lo cubre DIST-04); verificar en el Release PR #238 que el bump de `vantadb` actualice el requirement `version = "0.8.0"` del dep (si release-plz no lo hiciera, el PR no compila y se corrige ahí).

## Deuda técnica (Regla 6)

Sin deuda. Cambio de metadata/manifiesto, sin código nuevo. Nota: el requirement `version = "0.8.0"` es deuda *de sincronización* esperada (la gestiona release-plz); se registra en Invariantes/Notas, no como deuda P2.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato 1-4 ✅ (dry-run verde + smoke tmp + hold registrado + workspace intacto) + fmt scoped + gates docs |
| **Commit** | Commits atómicos `chore(release):` / `docs(tasks):`, verificación mecánica (nunca auto-reporte), sin archivos fuera del blast radius |
| **Release** | n/a — el publish real es F6/owner al cierre del plan (justificado; este task solo desbloquea). Sin changelog (el crate no publica aún) |

## Herramientas necesarias

- `cargo publish --dry-run` (sin token — jamás `--token` ni publish real)
- `cargo metadata` / `cargo tree -p vanta-memory`
- `codegraph_codegraph_explore` + `codebase-memory-mcp` (coverage/architecture)
- `campaign_verify_cmd` (checks mecánicos)
- `pwsh dev-tools/ocr-review.ps1` (cierre)
- python `tomllib` (validación TOML)

**Skills cargadas (SDP):** `ci-cd-and-automation` (pinned) · `git-workflow-and-versioning` (pinned) · `shipping-and-launch` (keyword-mapped release) · `coordinated-web-search` (research de docs oficiales) · `documentation-skill` (task file + .md) · `doubt-driven-development` (decisión release). Base auto (campaign-executor, progreso). SDP v3 excluyó por score/dominio: `incremental-implementation`, `test-driven-development`, `context-engineering` (sin lógica nueva ni tests de código; el smoke es verificación manual).

## Investigation Notes

- **release-plz — semántica de exclusión (docs oficiales):** `[workspace] publish = true` publica todos; `[[package]] publish = false` salta `cargo publish` **pero sigue creando tags**; `release = false` **ignora el paquete por completo** (update/changelog/tag/publish). Fuente: https://release-plz.dev/docs/config (§publish / §release). Precedente en repo: `vantadb-wasm` usa `release = false`.
- **release-plz + `publish = false` en Cargo.toml:** el paquete no-publicable se saltea (doc §git_only: "Packages with `publish = false` ... are **also** released" solo en git_only). Evidencia en repo: el release 0.8.0 (merge `ef2e33bd`) NO tocó `vanta-memory` (sin bump/changelog) → comportamiento actual confirmado.
- **crates.io Trusted Publishing (prereq):** "Your crate must already be published to crates.io (**initial publish requires an API token**)" + "Configure your crate on crates.io: Settings → Trusted Publishing" → la primera publicación de `vanta-memory` NO puede ser OIDC; requiere bootstrap del owner. Fuente: https://crates.io/docs/trusted-publishing (vía r.jina.ai, verificado 2026-10-04).
- **Cargo — path deps y publish:** "Crates that use dependencies specified with only a path are not permitted on crates.io" → `version` obligatorio junto a `path`. Dev-deps path-only: "only dev-dependencies that specify a `version` will be included in the published crate" (se excluyen, sin error). Fuente: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html
- **Cargo — metadata:** campos requeridos: `license`/`license-file` + `description`; recomendados: `homepage`, `repository`, `readme`. Fuente: https://doc.rust-lang.org/cargo/reference/publishing.html
- **Estado crates.io (verificado 2026-10-04):** `vantadb` 0.8.0 publicado (2026-10-02, trustpub OIDC); `vanta-memory` **no existe** (404 → "invisible" confirmado).
- **Baseline dry-run (pre-cambio):** `cargo publish --dry-run -p vanta-memory` → `error: vanta-memory cannot be published. package.publish must be set to true or a non-empty list` (exit 101) — bloqueante #1 confirmado.
- **Riesgo residual del dry-run:** la verificación compila contra `vantadb 0.8.0` de crates.io (no contra el árbol local). `MemoryInput`/`MemoryListOptions` de 0.8.0 ya exponen los campos usados (`sparse_vector`, `ttl_ms`, v2 fields) — verificado en docs.rs source. El dry-run es el test autoritativo.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — la decisión release-plz quedó resuelta por evidencia (hold); el dry-run es ejecución |
| Pendientes de ejecución (downhill) | 1 step (cierre: review + commits) |
| % completado | 83% (steps 1-5 ✅) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- **SECURITY:** no toca trust boundaries ni código; no agrega/quita dependencias (solo declara `version` de una dep existente). Superficie nueva = crate público (supply chain): mitigado con license Apache-2.0 + `cargo-deny` vigente + sin publish real. No aplica `security-and-hardening`.
- **PERFORMANCE:** no toca hot paths. No aplica.

## Steps

### Step 1 — Evidencia baseline + discovery — ✅
- **Archivos:** (lectura) `vanta-memory/Cargo.toml`, `release-plz.toml`, `release.yml`, `lib.rs`, historial git, crates.io API, docs oficiales
- **Acción:** confirmar bloqueantes, semántica release-plz/trusted-publishing, blast radius, estado crates.io
- **Verify:** baseline `cargo publish --dry-run` exit 101 con `publish=false`; `git log --all --grep="release v0.8"` sin vanta-memory; API crates.io 404 para vanta-memory
- **Estado:** ✅

### Step 2 — `vanta-memory/Cargo.toml` publicable + dry-run verde (core)
- **Archivos:** `vanta-memory/Cargo.toml`
- **Acción:** quitar `publish = false`; agregar `license`, `repository`, `homepage`, `documentation`, `keywords`, `categories`; `vantadb` dep → `{ path = "../", version = "0.8.0", default-features = false }`
- **Verify:** ✅ `cargo publish --dry-run -p vanta-memory --allow-dirty` exit 0 — "Packaged 139 files, 1.2MiB (328.7KiB compressed)"; el build de verificación compiló `vantadb v0.8.0` desde crates.io (manifest empaquetado: path dep reescrito a `version = "0.8.0"` + `default-features = false`); `Cargo.lock` sin cambios
- **Estado:** ✅

### Step 3 — Decisión release-plz (hold) registrada
- **Archivos:** `release-plz.toml`
- **Acción:** agregar `[[package]] name = "vanta-memory"` con `release = false` + comentario (bootstrap → trusted publishing → flip); coordinación #238
- **Verify:** ✅ `python -c tomllib` exit 0; entrada `[[package]] name = "vanta-memory"` presente con `release = false`
- **Estado:** ✅

### Step 4 — Doc-sync `lib.rs` (nota Stability)
- **Archivos:** `vanta-memory/src/lib.rs`
- **Acción:** actualizar la nota que dice "internal workspace member (`publish = false`)" → publicable + coordinación owner (sin tocar `docs/api/VANTA_MEMORY.md` — DIST-04)
- **Verify:** ✅ `cargo fmt -p vanta-memory --check` exit 0
- **Estado:** ✅

### Step 5 — Smoke externo (proyecto tmp)
- **Archivos:** (tmp) `%TEMP%\dist01-smoke\{Cargo.toml,src/main.rs}`
- **Acción:** proyecto cargo nuevo con `vanta-memory` como path dep; usar API pública (`name()` + `core::hooks::RecallConfig::default()`); `cargo check`
- **Verify:** ✅ `cargo run` exit 0 en `%TEMP%\dist01-smoke` — output: `dist01-smoke OK: vanta-memory loaded as external dep (recall max_results=5)`
- **Estado:** ✅

### Step 6 — Cierre: gates docs + OCR + review P2-01 + commits
- **Archivos:** `docs/dev/tasks/DIST-01.md` (+ los anteriores)
- **Acción:** gates docs (check-links/check-docs/gen-index); OCR delegation; review por agente distinto; commit local `chore(release):`; commit `docs(tasks):` con RESULTADO
- **Verify:** gates exit 0; OCR sin Critical/High; review APPROVE registrado; `git log` con commits locales (sin push)
- **Estado:** ⬜ PENDING

## Dependencias

- **Bloqueantes:** ninguno (F0).
- **Dependientes:** Task 17 (DIST-04) depende de DIST-01/02/03 (este task ejecuta la parte de `vanta-memory`).
- **Coordinación:** release 0.9.0 / #238 (decisión del owner; el hold de este task la hace explícita). `nextTask`: DIST-02.

## Review (GATE — agente distinto, P2-01)

> Pendiente (Step 6). Tier risk-based: paths tocados (`vanta-memory/Cargo.toml`, `release-plz.toml`, `vanta-memory/src/lib.rs` doc-comment, `docs/dev/tasks/DIST-01.md`) NO matchean globs adversariales → tier **fast**; se eleva a review fresco `vanta-review` (decisión de release, valor > mínimo).

- **Revisor:** ⬜ pendiente
- **Enfoque:** ⬜
- **Veredicto:** ⬜

## Notas

- **Decisión release-plz (hold):** el plan permite "entra en el próximo release **o** excluido explícitamente". Se eligió **hold** porque la ruta "entra" está mecánicamente rota HOY: el publish automático vía OIDC falla para un crate inexistente (prereq de crates.io) y rompería el job de release. El hold preserva el comportamiento actual (verificado en historial) y el unblock queda como checklist explícito del owner en ventana #238.
- **Unblock checklist (owner):** (1) primera publicación con API token temporal (`cargo publish -p vanta-memory` en árbol limpio o workflow one-off); (2) crates.io → crate → Settings → Trusted Publishing (repo `ness-e/Vantadb`, workflow `release.yml`); (3) borrar la entrada `[[package]] vanta-memory` de `release-plz.toml` → release-plz gestiona el crate (primer Release PR = release inicial; `max_analyze_commits` default 1000).
- **Sin README:** el crate no tiene README hoy; se difiere a DIST-04 (superficie documental del crate). Anotado como deuda.
- **Push:** diferido al final del plan (instrucción owner 2026-10-04) — commits locales.
