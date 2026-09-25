# FIND-66 — Formula sync (ARM64 + head + mcp)

> **Plan:** `docs/dev/plans/2026-09-15-find-correcciones.md` (Task 19, Wave6)
> **Estado:** ⬜ PENDING → 🟡 DISCOVERY DONE → ⬜ STEP-1 PENDING
> **Appetite:** 2h · 🟢 · 🟠 · **Ruta:** vanta-docs
> **Archivos clave:** `Formula/vantadb.rb`, `Formula/README.md`
> **SDP:** campaign-executor, progreso (base) + documentation-and-adrs, writing-guidelines (§6 rol vanta-docs) + lifecycle BUILD (incremental-implementation, source-driven-development — resto SDP descartado por irrelevante: frontend-ui-engineering, api-and-interface-design, test-driven-development, context-engineering, doubt-driven-development no aplican a sync docs↔rb)

## Contrato

- README veraz en los 3 puntos (ARM64 ✅ + head + mcp) + rb intacto en shas.
- Sin Ruby en runner → validación por lectura + diff revisado; `git diff --check` limpio.
- Prohibidos (NO TOCAR): `.opencode/`, `completions/`, `desktop/src-tauri/Cargo.lock`, stash@{0} GOV-C4, `vantadb-wasm/`, `integrations/dspy/`, shas del rb, `Cargo.toml`, workflows.

## Discovery — type & blast radius

- `campaign_detect_task_type` → `unknown` (docs-only sync, sin código).
- codegraph_explore NO aplica (Ruby/docs, no indexado → lectura directa + grep, según briefing).
- Blast radius: **1 archivo editado** (`Formula/README.md`); `Formula/vantadb.rb` solo lectura; tarball release solo lectura (evidencia).
- Gate D: NO disparado — blast radius 1 archivo, contrato no ambiguo, 0 símbolos públicos nuevos, 0 hot paths.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `Formula/vantadb.rb` (52 líneas), `Formula/README.md` (58 líneas), `.github/workflows/release-binaries-63.yml` (líneas 113-184, sección Package).
- **Referencias hacia dentro (qué cita el README):** `vantadb.rb` (shas, binarios instalados), releases page (URL), `brew` CLI.
- **Referencias entrantes (quién cita estos archivos):** `rg vantadb-mcp Formula/` → solo `Formula/README.md:24` (fila stale); `rg "Package binaries" workflow` → tarball `vanta-cli + vantadb-server`; ningún workflow/Cargo cita el README del tap.
- **Veredicto:** edición segura acotada a `Formula/README.md`; `vantadb.rb` NO se edita (shas intactos, stanza head NO se añade — decisión Spec H2).

## Verificación línea por línea (3 mismatches vs rb real)

| # | README (stale) | rb real (fuente) | Veredicto |
|---|---|---|---|
| M1 ARM64 | `:56` macOS ARM64 "🚧 Planned"; tabla sin Linux ARM64; `:28` Requirements "Linux (x86_64)" | `:24-27` stanza `on_arm` macOS + `:35-38` stanza `on_arm` Linux — SIRVE aarch64 en ambos OS | README miente; corregir tabla + requirements |
| M2 head | `:40` documenta `brew install --head` + nota "builds from source via cargo" | rb SIN stanza `head` (leído completo, 0 ocurrencias) → `--head` falla | Gap real; quitar doc (H2) |
| M3 mcp | `:22-24` tabla lista 3 binarios incl. `vantadb-mcp` | `:41-47` `def install` instala exactamente 2 (`vanta-cli`, `vantadb-server`); comentario `:42-44` confirma tarball trae esos 2 (+ ponytail: mcp futuro) | README miente; quitar fila mcp (H3) |

**Evidencia tarball (fuente tercera):** `release-binaries-63.yml:124` → `tar czf vantadb-${{ matrix.target }}.tar.gz -C release vanta-cli vantadb-server` (exactamente 2, sin mcp).

## Spec (decisiones)

| ID | Decisión | Alternativa descartada | Por qué |
|---|---|---|---|
| H1 | Marcar macOS ARM64 ✅ + añadir fila Linux ARM64 ✅ + requirements "Linux (x86_64 or ARM64)" | Dejar "Planned" | rb sirve aarch64 en ambos OS (evidencia M1); doc debe reflejarlo |
| H2 | QUITAR sección `## Local development` (`--head`) del README; NO añadir stanza `head` al rb | Añadir `head "...git", branch: "main"` al rb | Sin Ruby/`brew audit` en runner no se puede validar sintaxis DSL nueva (pre-mortem plan); añadir stanza sin verificar violaría Regla 11 (0 claims sin fuente); quitar doc = diff mínimo, riesgo 0, shas intactos |
| H3 | QUITAR fila `vantadb-mcp` de la tabla What's installed; NO instalar mcp en el rb | `bin.install "vantadb-mcp"` | Tarball no contiene mcp (evidencia workflow :124); instalar rompería la fórmula; el ponytail del rb (`:44`) ya anticipa el futuro wiring |

## Steps atómicos

- [x] **STEP-1 — README sync 3 puntos** (~30 líneas, 1 archivo): aplicar H1+H2+H3 en `Formula/README.md`; verify mecánico: `git diff --check` + `git diff Formula/` revisado línea por línea + shas rb intactos (`git diff --stat` solo README) + lectura final del README completo. Commit `docs: FIND-66 — Formula sync (ARM64 + head + mcp)`.

## Context Save Point

- Discovery completo 2026-09-15; decisión H2/H3 = quitar-doc (no añadir código Ruby sin runtime); rb verificado intacto (shas :22,26,33,37 no tocados); WIP ajeno en `.opencode/`, `completions/`, tauri lock, `docs/pipeline-state.json` — NO incluir en commit (solo `Formula/README.md`).
