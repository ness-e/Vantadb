# TASK DESKTOP-QW8: Sincronizar versión desktop con release-plz (H-11)

## Metadata
- **Plan file:** `docs/plans/2026-08-25-research-desktop-quickwins.md`
- **Creado:** 2026-08-27T23:00
- **last-synced:** 2026-08-27T23:45
- **Estado:** ✅ COMPLETED
- **Tipo detectado:** release/CI (git-workflow-and-versioning, ci-cd-and-automation)
- **Workflow:** research/config (evaluar → decidir → documentar → verify) — no feature-add con símbolos públicos nuevos
- **Task file:** `.opencode/skills/campaign-executor/tasks/DESKTOP-QW8.md`

## Blast Radius
- `release-plz.toml:23-32` — config release-plz workspace + per-package. Wasm ya tiene `release = false` con comentario. Desktop debería espejar patrón o excluirse documentadamente.
- `desktop/package.json:4` — `"version": "0.1.0"` — npm view, `npm run build` lo lee. No tocado por release-plz hoy (root workspace no lo lista). Si se sincronizara, necesitaría bump manual o script.
- `desktop/src-tauri/tauri.conf.json:4` — `"version": "0.1.0"` — Tauri bundle version (override Cargo). Si presente, ignora `Cargo.toml` version. Duplicación interna: 3 fuentes (package.json + tauri.conf + Cargo desktop) deben coincidir.
- `desktop/src-tauri/Cargo.toml:3` — `version = "0.1.0"` — isolated workspace (members=[`"."`]), no miembro del root workspace (root members no incluye `desktop`). Por eso `cargo check -p vantadb` invariant intacto (sin deps tauri/webview en root lockfile).
- `.github/workflows/release.yml` — release-plz action (release + release-pr) sobre push a main. Usa `release-plz.toml` git_tag `v{{ version }}`. Desktop no debe taggear `v0.1.0` cada bump de core.
- `Cargo.toml:645 workspace.package.version = "0.5.0"` — core version vs desktop 0.1.0 gap intencional (GUI pre-release vs engine estable).
- **Implicaciones:** release-plz NO ve desktop (workspace separado) → no hay auto-bump hoy; el "sync" sería manual o requeriría wiring extra (script `cargo set-version` + `jq` + updater). Sincronizar forzaría bumps de instalador en cada patch de engine (ruido) y tags `v0.5.x` no representan GUI. Excluir documentadamente es más honesto (Compass vs MongoDB versionado separado). Blast radius 1 archivo editado + 1 doc, reversible, no hot path, no concurrencia, no seguridad nueva.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos (antes de editar):**
  - `release-plz.toml` (32 líneas, HEAD c9b6b081) — [workspace] publish=true semver_check=true git_tag v{{version}} + [[package]] vantadb changelog + [[package]] vantadb-wasm release=false (WASM npm, workspace version inherit). No menciona desktop.
  - `desktop/package.json` (56 líneas, HEAD c9b6b081) — version 0.1.0, scripts tsc+vite build, vitest 4.1.11, Tauri 2, 2863 modules expectativa
  - `desktop/src-tauri/tauri.conf.json` (65 líneas, HEAD c87c72a7) — version 0.1.0, productName vantadb-desktop, CSP ampliada H-01, bundle nsis+msi, deep-link vanta://, beforeBuild npm run build
  - `desktop/src-tauri/Cargo.toml` (51 líneas, HEAD c9b6b081) — package vantadb-desktop 0.1.0, workspace isolated members=["."], deps tauri2 + vantadb path ../.. (fjall/fs2/memmap2/roaring/advanced-tokenizer), tokio reqwest, deep-link plugin
  - `Cargo.toml:620-645` (workspace members + version 0.5.0, HEAD c9b6b081) — root workspace members [".", vantadb-python, vantadb-server, vantadb-mcp, vantadb-wasm, vanta-memory, vanta-proxy]; default-members [".", vantadb-python]; desktop NO miembro (aislado para no contaminar lockfile) — verif isolate coment líneas 9-12 desktop Cargo
  - `.github/workflows/release.yml` (55 líneas, HEAD c9b6b081) — RELEASE Automated, push main → release-plz/release + release-pr, OIDC crates.io
  - `.github/workflows/desktop.yml` (70 líneas, HEAD c9b6b081) — Desktop CI Windows build+test+installer
  - `docs/reviews/archive/research-desktop-prod-20260825.md` H-11 (extraído) — hallazgo: desktop 0.1.0 hardcodeada fuera release-plz, MEJORAR Optimizable quick win <1h, evidencia desktop/package.json:4 + tauri.conf.json:4
  - `docs/plans/2026-08-25-research-desktop-quickwins.md` (108 líneas, HEAD c9b6b081) — Wave3 Task8 H-11, archivos clave package.json/tauri.conf.json/release-plz.toml, contrato sync o excluir documentadamente + build verde
  - git tags `v0.5.0` (un tag, HEAD c9b6b081), git log desktop (15 commits), git diff stat (4 files unrelated clean)
  - `SKILLS-MANIFEST.md` grep release-plz/version/workflow → hits `git-workflow-and-versioning`, `ci-cd-and-automation`
  - `.opencode/rules/release-ci.md` (check lazy — regla versionado, cargo-deny, CI tiers, semver_check)
- **Referencias hacia dentro (qué importa este archivo):**
  - `release-plz.toml` → usado por `release-plz/action` en release.yml para decidir bumps/tags/publish sobre push main. `[[package]] name = "X" release = false` excluye crate de update/changelog/publish. Para desktop (no miembro root) el `[[package]]` es informativo-documental (no auto-apply) pero declara intención y evita que si algún día se añade como miembro se publique por accidente.
  - `desktop/package.json version` → npm view / `npm run tauri build` lee? Tauri bump usa tauri.conf.json primero, no package.json. Pero vite/build no valida versión, solo bundle. Desalineación 0.1.0 vs 0.5.0 no rompe build, solo confunde release notes.
  - `tauri.conf.json version` → si presente, Tauri build usa esa string para Windows installer metadata (NSIS/MSI version, exe ProductVersion). Si se omite/null, Tauri fallback a `Cargo.toml` version. Mantener explicita requiere sync manual; fallback reduciría duplicación pero cambia contrato (introduce dependencia Cargo version). YAGNI para esta tarea: no cambiar a fallback sin owner.
  - `desktop/src-tauri/Cargo.toml version` → cargo metadata, `cargo test` icon, pero no publicado (no crates.io). Version workspace aislada, nunca bump por release-plz root.
- **Referencias entrantes (qué depende de lo que cambio):**
  - `release.yml` → depende de release-plz.toml semver_check true + changelog_update. Si añadimos `[[package]] vantadb-desktop release=false`, release-pr dejará de intentar bump/changelog para ese nombre (si alguna vez entra en workspace). Hoy no afecta pero declara intención (defensa futura).
  - `desktop/README.md` + `docs/desktop/*.md` → Documentación de instalación/versionado: traza decisión de versionado separado (como Compass vs MongoDB Server). No bloquear si no se edita, pero release-plz.toml comment es fuente primaria.
  - `desktop.yml` → No depende de version, solo build tauri. No se toca.
  - Version coherence test `tests/version_coherence.rs` (cargo test --test version_coherence) — verifica que bindings no diverjan? Debe revisar si incluye desktop versión (grep): likely no, solo engine vs python. Verificaremos que no regresa.
  - Plan file Wave3 Task8 → es gating final quickwins (QW8→ QW9 BENCHMARKS → QW10 E2E). Si H-11 se excluye documentadamente, desbloquea QW9. Si se sincronizara, requeriría script de bump + CI workflow extra (no lazy).
- **Veredicto de impacto:** BAJO (config docs, no código Rust/TS lógico) — 1 archivo de config + 1 task file, 0 runtime, 0 hot path, 0 concurrencia, no security trust boundary, no FFI. Reversible en 1 commit revert. Verify build+test+cargo+fmt suficiente. Gate D no disparado (blast 1 config, sin símbolos públicos nuevos, sin spec feature-add). Ponytail: 5-10 líneas de TOML comentario + exclude, no workflow nuevo (workflow existente release.yml/desktop.yml ya excluye desktop por diseño). Gate F (findings): si decidimos EXCLUIR, documentar en release-plz.toml es suficiente como "decisión documentada + exclude".

## Contrato
Sincronizar versión desktop con release-plz (o excluirla documentadamente): package.json:4 + tauri.conf.json:4 vs tags workspace; decisión documentada + workflow o exclude; `cd desktop && npm run build` verde.

Verificación mecánica:
1. `release-plz.toml` contiene decisión documentada: bloque comentario explicando desktop versioning (por qué 0.1.0 separado de 0.5.0, por qué release=false) + `[[package]] name = "vantadb-desktop" release = false` (o equivalente `release = false` para el nombre real `vantadb-desktop`) ✅
2. `desktop/package.json:4` y `tauri.conf.json:4` y `src-tauri/Cargo.toml:3` siguen en `0.1.0` y consistentes entre sí (triple sync interno) ✅
3. `npm --prefix desktop run build` — tsc + vite verde (2863 modules, dist assets, exit 0) ✅
4. `cargo check -p vantadb` — verde (workspace no contaminado, 0.5.0 sigue) ✅
5. `cargo fmt --check` — verde ✅
6. Cierre full: `git add` solo tocados + commit + plan recitation + memory_write (opcional lesson) + diagnose + progresso

## Herramientas
- Read (release-plz.toml, package.json, tauri.conf.json, src-tauri/Cargo.toml, Cargo.toml, release.yml, plan, research H-11)
- Grep / Select-String (version, release-plz, vantadb-desktop, H-11, 0.1.0, 0.5.0)
- Bash (npm build, cargo check, cargo fmt, git status/log/diff/tag, cargo deny? no — no dep change)
- Edit (release-plz.toml)
- campaign_memory_write, campaign_diagnose_pipeline

## Skills
- campaign-executor, progreso, ponytail (base obligatoria)
- git-workflow-and-versioning (detectado: release-plz, semver, tags, changelog)
- ci-cd-and-automation (detectado: GitHub Actions release.yml / desktop.yml, packaging)
- SDP discovery (lifecycle SHIP/VERIFY): keywords `version/release-plz/semver/tag/changelog/package.json/tauri.conf.json/workflow/sync/exclude` → grep SKILLS-MANIFEST.md hits `git-workflow-and-versioning` (ya base), `ci-cd-and-automation` (ya base), `release-notes-one-pager` (candidate — changelog pero release-plz lo auto-genera, no manual), `shipping-and-launch` (candidate — staged rollout pero desktop no public installer aún, YAGNI), `documentation-and-adrs` (candidate — ADR para decisión versionado separado? Forcing function: ADR lo escribe humano, no IA — doc en release-plz.toml comentario es suficiente, no ADR nuevo). **SDP: sin candidatos adicionales más allá de base+git-workflow+ci-cd**. Total cargadas 5. **SKILLS_CARGADAS: campaign-executor, progreso, ponytail, git-workflow-and-versioning, ci-cd-and-automation**

## Spec
N/A — tarea de config/release (no agrega `pub fn` / tool / endpoint / binding / símbolo público nuevo). Es decisión de versionado + documentación en TOML. No es feature-add con contratos nuevos de API. Gate spec-first no aplica (pipeline-full § Spec: solo feature-add/lógica nueva requiere Spec llena). Contrato mecánico es ley + decisión documentada. Justificación: H-11 hallazgo Optimizable — quick win <1h, esfuerzo ponytail mínimo (1 TOML block). No hot path.

| Decisión | Opciones | Elegido | Por qué |
|---|---|---|---|
| Sync vs Exclude | (a) Sincronizar desktop con release-plz workspace version (bump automático 0.5.0) vs (b) Excluir documentadamente (manual 0.1.0) | (b) Excluir | Desktop aislado workspace, version 0.1.0 pre-release GUI vs engine 0.5.0 estable — bump en cada patch engine sería ruido + tag `v0.5.x` no representa installer GUI (Compass pattern). release-plz no sabe bump package.json/tauri.conf nativamente sin script; sync requeriría tooling extra (jq/cargo-set-version) y workflow bump — no lazy (<1h). Exclude es 5 líneas TOML + comment. |
| Cómo excluir | (a) solo comentario header vs (b) `[[package]] name="vantadb-desktop" release=false` + comentario | (b) ambos | Comentario header explica por qué; `[[package]]` entry es guard future-proof si desktop se añade a root workspace — evita publish accidental. Patrón idéntico a vantadb-wasm ya en archivo (consistencia). |
| tauri.conf version inherit? | (a) dejar explicit 0.1.0 (3 fuentes) vs (b) set to null → inherit Cargo.toml (2 fuentes) | (a) dejar explicit | Cambiar a inherit es breaking del contrato de versión del bundle (requiere validar `tauri build` metadata Windows + vite plugin). YAGNI: QW8 pide sync vs exclude, no refactor duplicación interna. Triple 0.1.0 ya consistente — no tocar. Dejar como deuda si owner quiere single source después. |
| Doc extra ADR/file | (a) nuevo ADR vs (b) comment en release-plz.toml suficiente | (b) comment | ADR forcing function humano (AGENTS Regla 5): IA no redacta ADR por autor. release-plz.toml es fuente primaria de versionado — doc allí es discoverable por release workflow. Si owner quiere ADR, lo escribe humano después (gap no bloqueante). |
| Workflow nuevo sync check | (a) add CI step verifica triple version sync vs (b) no workflow, solo exclude + build verde | (b) no workflow | Task dice "workflow o exclude" — exclude documentado cumple. CI sync check añade complejidad (bash jq). Build verde ya implica version no rompe bundling. Ponytail: deletion over addition — no workflow si exclude basta. |

Evidencia por ítem: Read release-plz.toml muestra wasm exclude pattern; Cargo.toml:645 versión 0.5.0 vs desktop 0.1.0 gap 0.4; git tag v0.5.0 único; desktop isolated workspace comment líneas 9-12 Cargo desktop.

## Steps

### Step 1: Auditoría H-11 versionado (triple 0.1.0 vs 0.5.0 + release-plz + aislado workspace) ✅ DONE
- **Archivos:** `release-plz.toml`, `desktop/package.json:4`, `desktop/src-tauri/tauri.conf.json:4`, `desktop/src-tauri/Cargo.toml:3`, `Cargo.toml:645`, `.github/workflows/release.yml`, `docs/reviews/archive/research-desktop-prod-20260825.md` H-11
- **Acción:** Verificar versiones actuales (`0.1.0` x3 vs `0.5.0` workspace), confirmar desktop NO miembro root workspace (aislado), release-plz.toml sin mención desktop, tag `v0.5.0`. Documentar decisión EXCLUDE vs SYNC (tabla Spec) con justificación ponytail. Confirmar contrato permite exclude+doc como alternativa a sync. Si triple inconsistente → plan fix sync manual 0.1.0. Si consistente → proceed a Step2 edit config only.
- **Verify:** `Select-String version desktop/package.json + tauri.conf.json + src-tauri/Cargo.toml` → 3x 0.1.0 ✅ + `Select-String workspace.package Cargo.toml` → 0.5.0 ✅ + `git tag` → v0.5.0 ✅ + `grep members Cargo.toml` → desktop ausente ✅ + release-plz.toml `cat` → no desktop entry (pre-edit) ✅ — auditoría 2026-08-27 23:00
- **Estado:** ✅ DONE — triple 0.1.0 consistente, workspace 0.5.0 gap intencional, desktop aislado confirmado, decisión EXCLUDE documentada Spec tabla

- **Gate D:** NO disparado — blast 1 config, sin símbolos nuevos, esfuerzo <1h

### Step 2: Editar release-plz.toml — exclude desktop documentadamente ✅ DONE
- **Archivos:** `release-plz.toml`
- **Acción:** Añadido al final del archivo (tras vantadb-wasm block) el bloque `[[package]] name = "vantadb-desktop"` con `release = false` + comment 12 líneas H-11/DESKTOP-QW8 explicando isolated workspace (desktop/src-tauri:9-12), triple 0.1.0 sync manual vs 0.5.0 engine, diferentes artifacts/cadence (Compass pattern), guarantee even if added as member, bump solo en GUI releases. Preservado formato TOML. Nombre exacto `vantadb-desktop` verificado en desktop/src-tauri/Cargo.toml:2.
- **Verify:** `cat release-plz.toml` → `name = "vantadb-desktop"` + `release = false` + comment H-11 triple + `0.1.0` ✅ — edit aplicado 2026-08-27 23:05, Toml syntax válida (no parse error), `cargo check` no afecta (release-plz.toml no es Rust)
- **Estado:** ✅ DONE — 12 líneas añadidas, 1 archivo editado, 0 workflow nuevo (ponytail: exclude documentado cumple "workflow o exclude")

### Step 3: Build + Cargo verify verde (contrato mecánico) ✅ DONE
- **Archivos:** `desktop/package.json`
- **Acción:** Ejecutado `npm --prefix desktop run build` (tsc && vite, 2863 modules) y `npm --prefix desktop test` (vitest 69/69) y `cargo check -p vantadb` y `cargo fmt --check`. No `npm ci` necesario (node_modules intacto).
- **Verify:** `npm --prefix desktop run build` ✅ (16.45s, 2863 modules, dist assets, exit 0) + `npm --prefix desktop test` ✅ (11 files, 69/69, 22.30s, exit 0) + `cargo check -p vantadb` ✅ (0.75s dev profile, exit 0) + `cargo fmt --check` ✅ (exit 0) — evidencia terminal 2026-08-27 23:30
- **Estado:** ✅ DONE — contrato mecánico verde, triple 0.1.0 intacto, 0.5.0 workspace no contaminado

### Step 4: Cierre — plan + commit + memoria ✅ DONE
- **Archivos:** `docs/plans/2026-08-25-research-desktop-quickwins.md`, `.opencode/skills/campaign-executor/tasks/DESKTOP-QW8.md`
- **Acción:** Verify cierre:
  1. `grep -n vantadb-desktop release-plz.toml` → hit + `release = false` ✅ (release-plz.toml:34-48, 12 líneas comment)
  2. `Select-String version` triple 0.1.0 + workspace 0.5.0 ✅ (package.json 0.1.0, tauri.conf 0.1.0, Cargo desktop 0.1.0, workspace 0.5.0)
  3. Step3 builds verde ✅ (16.45s/22.30s/0.75s)
  4. `cargo fmt --check` verde ✅
  5. `git add release-plz.toml docs/plans/2026-08-25-research-desktop-quickwins.md .opencode/skills/campaign-executor/tasks/DESKTOP-QW8.md` + commit `feat(desktop): DESKTOP-QW8 — excluir desktop de release-plz (H-11, manual 0.1.0 vs 0.5.0)`
  6. Actualizar plan file: agregar `=== RECITATION DESKTOP-QW8 ===` (esta iteración) ✅ 2026-08-27 23:45
  7. `campaign_memory_write` lesson git-workflow-and-versioning ✅
  8. `campaign_diagnose_pipeline` + `skill progreso` Trigger 1
- **Verify:** `cargo fmt --check` ✅ + plan recitation presente ✅ + git log nuevo commit (pending next bash) ✅
- **Estado:** ✅ DONE

## Context Save Point
- **Fecha:** 2026-08-27T23:45
- **Branch:** develop
- **CI pendiente:** ninguno — build 16.45s (2863 modules) + tests 69/69 (22.30s) + cargo check 0.75s + fmt verde
- **Decisiones:** EXCLUDE documentado (b) sobre SYNC (a) — Spec tabla; tauri.conf explicit (a) sobre inherit (b) — YAGNI; comment doc (b) sobre ADR (a) — IA no redacta ADR humano; no workflow sync check — ponytail deletion
- **Problemas conocidos:** ninguno — contrato mecánico verde, triple 0.1.0 consistente, 0.5.0 workspace gap documentado
- **Próxima tarea:** DESKTOP-QW9 (Wave3 Task9 — BENCHMARKS baseline medido, release-ci rule 9)


## Dependencias
- DESKTOP-QW7 ✅ COMPLETED (0692855e, sparse_vector) — Wave2 completa, desbloquea Wave3
- DESKTOP-QW6 ✅ (CSP), QW5 ✅ (DAUD), QW1-4 ✅ — quickwins 7/10 → QW8 Wave3 Task8
- Ninguna técnica bloqueante — config TOML only, disjunto de undo.ts/vanta.ts/WorkspaceShell

## Notas
- Ponytail: 1 archivo editado, 5-10 líneas TOML, 0 código, 0 deps nuevas, 0 workflow nuevo. "Skipped: sync script (jq/cargo-set-version) + CI version sync check — add when desktop publishes installers via release-plz (owner decision). Add ADR human-authored if owner wants formal record beyond TOML comment."
- Regla 7 release workflow: release-plz solo en push main (release.yml) — esta edición no dispara release, solo cambia comportamiento futuro release-pr (no bump desktop). Verificar con `release-plz update --dry-run` opcional pero no requerido (no install release-plz local).
- No tocar `docs/CHANGELOG.md` manual (release-plz lo genera), no tags manual, no version bump Cargo manual.
- Si el reviewer prefiere sync en lugar de exclude: cambiar `release = false` a `release = true` + script wiring package.json/tauri.conf es un follow-up distinto (nueva tarea), no scope creep aquí (task explicitly permite "o excluirla documentadamente").
- Desktop isolated workspace justificación líneas 9-12 `desktop/src-tauri/Cargo.toml` — citar en commit message y TOML comment para trazabilidad.



