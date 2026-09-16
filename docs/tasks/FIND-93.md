# FIND-93 — `dead_code` `cloned_sole_buffer` en `src/storage/engine/txn.rs:158`

> Campaign: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0 · Wave9 (plan-adjust 2026-09-16) · Appetite 2h (🟢)
> Branch: develop · Commit: `fix: FIND-93 — ...` (solo archivos propios; push vía vanta-lead)
> Estado: ⏳ IN PROGRESS (DISCOVERY completo, fix pendiente)

## 1. TAREA

**Objetivo:** eliminar el error `dead_code` (`cloned_sole_buffer` never used,
`src/storage/engine/txn.rs:158`) que bloquea el gate `clippy -D warnings` de
cualquier crate que dependa de `vantadb` sin default features (verificado:
`vanta-proxy`).

**Contrato exacto:**
- `cargo clippy -p vantadb --all-targets -j 2 -- -D warnings` → verde (0 warnings,
  config default = rayon ON)
- `cargo check --workspace -j 2` → verde
- Tests del módulo storage/engine verdes
- `cargo clippy -p vanta-proxy --all-targets -j 2 -- -D warnings` → verde
  (desbloqueo verificado: era el bloqueo de FIND-65 y FIND-75)

**Acceptance criteria:**
1. Clippy `-D warnings` verde en `vantadb` (all-targets, default features).
2. Clippy `-D warnings` verde en `vanta-proxy` (all-targets) — el desbloqueo.
3. Clippy verde en `vantadb` SIN default features (la config que disparaba el lint).
4. Workspace check verde + tests del módulo verdes + `git diff --check` limpio.
5. Sin deuda neta (Regla 6): el fix NO usa `#[allow]`; no se añade deuda.

## 2. ARCHIVOS

**Clave (verificado en DISCOVERY):**
- `src/storage/engine/txn.rs:158` — `pub(crate) fn cloned_sole_buffer` (el método;
  el fix es 1 línea: `#[cfg(feature = "rayon")]` sobre el método, simétrico con su
  caller chain).

**Relacionados (callers/callees vía `rg` + codegraph_explore 2026-09-16):**
- `src/storage/engine/insert.rs:784` — `existing_for_batch_many`
  (`#[cfg(feature = "rayon")]`, ÚNICO caller no-test) ← `:547`
  `probe_existing_for_batch` (cfg rayon) ← `:563` `apply_batch_stats_rayon`
  (cfg rayon) ← `:896-897` rama cfg rayon de `batch_insert_with_opts`.
- `src/storage/engine/get.rs:203` — caller siempre-activo de `probe()` (no afectado;
  `probe` NO se toca).
- `src/storage/engine/tests/engine.rs:1388` — test de `probe_existing_for_batch`
  (cubierto bajo rayon default; no se toca).
- `vanta-proxy/Cargo.toml:29` — `vantadb = { path = "..", default-features = false }`
  (la config que expone el lint; no se toca).
- `Cargo.toml:106` — default incluye `rayon` (por eso el lint solo muerde sin defaults).

**Tests del módulo:** `src/storage/engine/txn.rs:671-765` (`txn_manager_tests`) +
`src/storage/engine/tests/` (engine.rs, incremental.rs).

**Prohibidos (WIP ajeno — NO tocar):** `.opencode/` (submodule, solo lectura),
`Justfile`, `completions/_vanta-cli*`, `desktop/src-tauri/Cargo.lock`,
`.github/workflows/ocr-delegate.yml`, `dev-tools/ocr-review.ps1`, `reparacion.bat`,
`docs/pipeline-state.json`, plan file (solo recitation orquestador), `stash@{0..14}`,
archivos de FIND-92 (`.github/workflows/ci-rustdoc.yml`) y FIND-94 (`integrations/`,
`vantadb-python/`), resto de `src/` fuera de `txn.rs` salvo lectura.

## 3. DEPENDENCIAS

Wave9 (plan-adjust 2026-09-16; 28/30 con FIND-92 ✅ e395b563). Sin bloqueantes.
Paralelas: FIND-92 ✅, FIND-94 (disjunta: `integrations/` + `vantadb-python/`).
Previa: FIND-72 ✅. Next: FIND-94, luego cierre (progreso masivo + retrospectiva + archive).

## 4. REFERENCIAS

- `.opencode/rules/core-engine.md` — LEÍDA COMPLETA. Aplica R-2 (funciones internas
  sin callers externos → no exportar; el método ya es `pub(crate)` ✅, el fix no
  cambia visibilidad) y R-1 (simetría de feature-gating: el fix alinea el callee
  con su caller chain bajo `rayon`). R-3/R-4/R-5 no aplican (sin unwrap/unsafe/env).
- `.opencode/rules/durability.md` — LEÍDA. txn vive en su scope (`storage/engine/`)
  pero el fix no toca WAL/backends/GC/LSM → sin implicaciones de durabilidad.
- `.opencode/references/clean-code-clean-architecture.md` — LEÍDA COMPLETA
  (incl. Apéndice V). Fix quirúrgico 1 línea; sin stuttering/naming/deuda nueva;
  Boy Scout + Regla 6: saldo neto ≤ 0 (cfg-gate, cero deuda).
- Regla 6 (deuda): NO se usa `#[allow]`/`#[expect]` → no hay deuda que pagar.
  `#[expect(dead_code)]` sería INCORRECTO: bajo default features el lint no dispara
  y `unfulfilled_lint_expectations` (deny by default) rompería el build con rayon.
- Gate D: NO dispara. El método es `pub(crate)` (API interna, no pública); el fix
  no añade/quita símbolos públicos ni cambia firmas. Eliminar el método NO es opción
  (rompería el build con rayon + regresión ERR-037); por eso no hay decisión
  breaking que cuestionar.
- Notion (Paso 0c, 4 páginas leídas 2026-09-16 vía fetch): `Problema`, `Propuesta`,
  `Nuevas features` (índice), `Plan de accion` (índice). Filtro VantaDB: ninguna hija
  mapea a este fix de lint (no es feature ni proposal); `Propuesta` §3/Anexo C
  confirma `storage/engine` como motor REAL (contexto, sin acción derivada).
- Propuesta §3 no aplica (no es feature).

## 5. SKILLS (SDP Paso 0b)

`campaign_discover_skills_v2` (phase=BUILD,
keywords=["dead-code","clippy-warnings","txn-storage","rust-lint"]) → 8 skills.
Cargadas efectivamente (6/8; `frontend-ui-engineering` descartada — sin `web/` en
scope; resto lifecycle genérico no aplicable a fix de 1 línea):

- `systematic-debugging` — bug con causa raíz no obvia ("nunca usado" vs caller
  cfg-gated); Phases 1-3 aplicadas inline (repro → caller chain → hipótesis cfg).
- `incremental-implementation` — 1 slice vertical (fix + verify + commit), scope
  discipline, compilable siempre.
- `test-driven-development` — Prove-It: RED = repro clippy proxy rojo (evidencia
  abajo); GREEN = cfg-gate + clippy verde ambas configs; sin test unitario nuevo
  (comportamiento intacto, tests existentes cubren ambas ramas).
- `context-engineering` — Selective Include por slice (rules → plan/recitation →
  txn.rs + insert.rs + Cargo.tomls → error clippy).
- `source-driven-development` — `#[cfg(feature)]` es lenguaje core (sin docs
  externas necesarias); `rustc --version` local (1.95.0) como evidencia, no web.
- `doubt-driven-development` — CLAIM adversarial: "cfg-gate es correcto y expect/
  allow/delete son incorrectos" (ver §7).
- `api-and-interface-design` — descarta Gate D (pub(crate), sin superficie pública).
- `SDP: systematic-debugging, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, api-and-interface-design`

## 6. HERRAMIENTAS + MCP

Comandos exactos (siempre `-j 2`; timeout generoso — clippy frío tarda):

```bash
cargo clippy -p vanta-proxy --all-targets -j 2 -- -D warnings   # RED (repro) + GREEN (desbloqueo)
cargo clippy -p vantadb --all-targets -j 2 -- -D warnings       # default features (rayon ON)
cargo clippy -p vantadb --all-targets -j 2 --no-default-features -- -D warnings  # config que mordía
cargo check --workspace -j 2
cargo test -p vantadb --lib storage::engine::txn -j 2           # tests módulo (o target que cubra txn)
git diff --check
rustc --version   # 1.95.0 (#[expect] estable desde 1.81 — irrelevante: no se usa)
```

MCP usados: `campaign_detect_task_type` (Rust core) ✅,
`campaign_discover_skills_v2` (BUILD) ✅, `codegraph_explore`
("cloned_sole_buffer txn insert batch" — blast radius: 1 caller) ✅,
`check_index_coverage` (proyecto `C-Users-Eros-VantaDB-Proyect-VantaDB`; paths —
`rg` como fallback verificado) ✅. `campaign_verify_cmd` — bug exit -1 conocido;
fallback bash directa y anotarlo en RESULTADO. `campaign_update_task_state`
(in-progress ✅ → completed al cerrar).

## 7. INVESTIGACIÓN CÓDIGO (blast radius — DISCOVERY 2026-09-16)

```
cloned_sole_buffer (txn.rs:158, pub(crate), sin cfg)
  └─ existing_for_batch_many (insert.rs:784, #[cfg(rayon)])   ← ÚNICO caller no-test
       └─ probe_existing_for_batch (insert.rs:547, #[cfg(rayon)])
            └─ apply_batch_stats_rayon (insert.rs:563, #[cfg(rayon)])
                 └─ batch_insert_with_opts (insert.rs:887, rama :896 #[cfg(rayon)])
probe() (txn.rs:177, sin cfg) ← get.rs:203 (siempre activo) + existing_for_batch
  (insert.rs:741, #[cfg(not(rayon))]) → NO afectado, NO se toca.
```

**Implicaciones:** allow vs remove vs cfg-gate → cfg-gate gana (ver §4).
**Riesgo:** nulo en API pública (pub(crate)); nulo en runtime (cero cambio de
comportamiento en ambas configs; la rama rayon compila idéntico código).

**Doubt cycle (CLAIM):** "Anotar `cloned_sole_buffer` con `#[cfg(feature =
"rayon")]` es el fix correcto; `#[expect]` rompería el build default,
`#[allow]` añadiría deuda Regla 6, y eliminar regresaría ERR-037."
WHY: decide entre 4 fixes con trade-offs reales en un hot-path de batch.
ARTIFACT: diff 1 línea + chain §7. CONTRACT: §1 acceptance 1-5.
Reconciliación: `expect` inválido (unfulfilled_lint_expectations es deny by
default → rojo con rayon); `allow` válido pero con deuda; delete inválido
(caller rayon existe). → cfg-gate es la única opción sin deuda y sin regresión.
Cross-model: skipped (contexto no-interactivo de tarea pipeline).

## 8. INVESTIGACIÓN PROBLEMA

El `dead_code` es PRE-EXISTENTE y condicional: solo muerde cuando `vantadb` se
compila sin la feature `rayon` (único caller cfg-gated). `vanta-proxy` depende con
`default-features = false` (sin rayon) → el gate `-D warnings` del proxy cae por
un lint en la dependencia. FIND-65 y FIND-75 lo sufrieron (repro stash-confirmado
en FIND-65: mismo error sin el fix; `vanta-proxy` solo = 0 warnings).

**Corrección a la premisa del Backlog :235 ("nunca usado"):** SÍ tiene caller
(`insert.rs:792`, path batch rayon ERR-037, default ON). La causa raíz es
asimetría de feature-gating (caller gated, callee no), no código muerto real.
(No se edita Backlog aquí — race paralelo, orquestador.)

## 9. INVESTIGACIÓN INTERNET

No requerida. `#[cfg(feature = "...")]` es Rust core estable; `rustc 1.95.0`
verificado local. `#[expect]` estable desde 1.81 [cita NO VERIFICADA — sin red en
esta sesión; irrelevante porque no se usa `expect`].

## 10. VALIDACIÓN + CIERRE

- Verify contrato (§1, comandos §6) + `git diff --check` + `cargo fmt --check`
  del file (o `cargo fmt -p vantadb -- --check`).
- OCR delegation: `pwsh dev-tools/ocr-review.ps1` (Critical/High = bloquea,
  Medium → FIND-*).
- `/cleanCA` sobre `src/storage/engine/txn.rs` antes de cerrar (norma ritual —
  audita, solo informa).
- DoD 3 niveles: contrato verde + scope discipline (diff = 1 línea + este task
  file) + recitation + RESULTADO §7.
- Reviewer distinto P2-01 (vanta-review; worker no se auto-audita).
- Gates: P no | D no (pub(crate), §4) | V no (salvo 2 fallas mismo-error) | C sí
  (colaterales: corrección premisa Backlog → informar, no tocar).
- Commit conventional `fix:` con task ID (solo `src/storage/engine/txn.rs` +
  `docs/tasks/FIND-93.md`). Backlog→avance NO tocar (orquestador) + push vía
  vanta-lead.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `src/storage/engine/txn.rs` (765L),
  `.opencode/rules/core-engine.md`, `.opencode/rules/durability.md`,
  `.opencode/references/clean-code-clean-architecture.md` (697L, Apéndice V incl.),
  `docs/plans/2026-09-15-find-correcciones.md` (recitations Wave9),
  `vanta-proxy/Cargo.toml` (dep sin defaults), `Cargo.toml` (features).
- **Leídos parcial:** `src/storage/engine/insert.rs` (:520-604, :700-830, :870-904),
  Notion ×4 (fetch completos).
- **Referencias hacia dentro (lo que el fix necesita):** feature `rayon`
  (`Cargo.toml:85,106,144`); caller chain cfg-gated (insert.rs :546,562,783,784).
- **Referencias entrantes (quién depende del método):** solo `existing_for_batch_many`
  + tests cfg-gateados; ningún otro caller en `src/` (`rg` 0 hits fuera de txn.rs +
  insert.rs:792 + docs/Backlog/tasks menciones).
- **Veredicto:** impacto mínimo y reversible (1 atributo cfg); compilable en ambas
  configs; sin cambios de comportamiento, firma, visibilidad ni docs públicas.

## Steps

- [x] Step 0 — DISCOVERY: repro + root cause + task file (este archivo)
- [x] Step 1 — Fix: `#[cfg(feature = "rayon")]` en `txn.rs:154-161` (+ nota doc 4 líneas)
- [x] Step 2 — Verify contrato (2026-09-16, bash directa — `campaign_verify_cmd`
  con bug exit -1 conocido):
  - `cargo clippy -p vantadb --all-targets -j 2 -- -D warnings` → ✅ exit 0 (34.77s)
  - `cargo clippy -p vanta-proxy --all-targets -j 2 -- -D warnings` → ✅ exit 0
    (21.39s; era rojo pre-fix — desbloqueo FIND-65/FIND-75 verificado)
  - `cargo check --workspace -j 2` → ✅ exit 0 (28.72s)
  - `cargo test -p vantadb --lib -j 2 storage::engine::txn` → ✅ 7/7
  - `cargo test -p vantadb --lib -j 2 storage::engine` → ✅ 383/383 (19.17s)
  - `cargo fmt -p vantadb -- --check` → ✅ exit 0; `git diff --check` → ✅ limpio
  - Colateral (NOTICED BUT NOT TOUCHING): `cargo clippy -p vantadb --all-targets
    --no-default-features` NO compila pre-existente (tests `security_audit` /
    `common/mod.rs:373` requieren `fs2`; config sin defaults nunca soportada).
    Fuera de scope, no se toca.
- [ ] Step 3 — Cierre: OCR + cleanCA + review P2-01 + commit + recitation + RESULTADO

## Context Save Point

Repro RED (2026-09-16, bash directa): `cargo clippy -p vanta-proxy --all-targets
-j 2 -- -D warnings` → `error: method cloned_sole_buffer is never used -->
src\storage\engine\txn.rs:158:19` + `could not compile vantadb (lib)`. Fix
decidido: cfg-gate (ver §7). Si se interrumpe: aplicar Step 1 y correr Step 2.
