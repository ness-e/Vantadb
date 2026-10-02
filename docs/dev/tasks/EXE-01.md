---
title: "EXE-01 - Demos CI (3 casos: memory/verify/governance)"
kind: task
---

# EXE-01: Demos CI (3 casos: memory/verify/governance)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 48, Fase F6)
- **Fuente:** plan Task 48 · Backlog P50:912 (DoD "3 demos verdes en CI + docs linkeadas")
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟠
- **Tipo:** Mixto (CI/DevOps + docs; **sin código de producto**)
- **Turns estimados:** 12-18
- **Creado:** 2026-09-30T01:46
- **last-synced:** 2026-09-30T03:28
- **Estado:** ⏳ IN PROGRESS (steps ✅ + micro-batch post-review ✅; commit = LANE LEAD por instrucción de wave)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps (S1–S6 ✅; cierre formal del LEAD)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | GitHub Actions (2 workflows demo nuevos, PR-blocking) · `docs/user/` (PRIVACY.md + AI_IDES.md → índices generados `docs/index.md`, `docs/user/index.md`, `llms.txt`) |
| Callees | `vanta-cli` (`verify`/`put`/`delete --attest`/`certificate verify` — superficies shipped VER-01/02) · `vanta-proxy` (injection budget/ACL/audit — VER-04 shipped) · `vantadb::wal::{WalHeader, WalRecord, WalWriter, compute_crc32c}` (fixtures del demo) |
| Implicaciones | 0 cambios de comportamiento de producto. `tests/wal_chain_verify.rs` y `vanta-proxy/tests/ver04_governance.rs` ganan 1 test `#[ignore]` c/u (no corren por default; los 4+4 existentes intactos); `seed_wal` de integración refactorizado internamente (misma firma). Docs + workflows nuevos. Sin migración, sin performance, sin wire. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `tests/wal_chain_verify.rs` (143L) · `vanta-proxy/tests/ver04_governance.rs` (330L) · `vanta-proxy/tests/icp02_privacy_demo.rs` (546L, patrón F5) · `vanta-proxy/src/governance.rs` (289L) · `vanta-proxy/src/inject.rs` (región :100-313) · `src/cli_handlers/wal.rs` (:283-423) · `src/cli.rs` (:1-268, regiones `Verify`/`Delete`/`Certificate`) · `src/audit.rs` (:18-199) · `src/wal.rs` (regiones unit tests :1820-1959) · `.github/workflows/{ci-ai-ides-demo,icp02-privacy-demo,ci-frameworks-demo,ci-examples,ci-gate}.yml` · `docs/user/PRIVACY.md` (168L) · `docs/user/AI_IDES.md` (:1-80) · `docs/api/WAL_INTEGRITY.md` (49L) · `.opencode/rules/release-ci.md` (42L).
- **Archivos referenciados hacia dentro (imports/deps):** `tests/wal_chain_verify.rs` → `vantadb::{config, node, wal}` + `tempfile` + postcard (ya presentes); `ver04_governance.rs` → `vanta_proxy::{config, server, cost}` + `vanta_memory` + `vantadb::{entity, sdk, storage}` + axum/reqwest/serde_json (ya presentes). Ningún import nuevo salvo `vantadb::node::FieldValue` (público, ya usado por otros tests).
- **Archivos que referencian a los editados (referencias entrantes):** `tests/wal_chain_verify.rs` ← mencionado en `docs/dev/tasks/VER-01.md` y plan (solo como evidencia; ningún import); `ver04_governance.rs` ← `docs/dev/tasks/ICP-02.md` (patrón, sin import); `docs/user/PRIVACY.md` ← `docs/index.md` + `docs/user/index.md` (generados) + `COMPARISON.md` + `docs/api/PROXY.md`; `docs/user/AI_IDES.md` ← `docs/user/index.md` + `llms.txt` (generados).
- **Veredicto impacto:** **bajo** — aditivo en tests (gated `#[ignore]`), refactor privado de un helper de test, docs y workflows nuevos. Nada se elimina; nada cambia de contrato público.

## Contrato

Contrato verbatim (plan Task 48):

> "3 demos ejecutables en CI con output documentado: **memory** (sesión 1 guarda → sesión 2 recupera; `ci-ai-ides-demo.yml`) Y **verify** (tamper en WAL → `vanta-cli verify` exit ≠0 + `delete --attest`/`certificate verify` exit 0 — CI rojo si no detecta) Y **governance** (budget/ACL/audit de inyección: fuera de ACL → `denied` en audit, bloque ≤ budget, consulta `op:"injection"`) Y cada demo con comando + output esperado documentado y linkeado desde `docs/user/` Y 0 `continue-on-error`"

Mapping por cláusula al artefacto (evidencia en §Notas al cierre):

| Cláusula | Artefacto |
|----------|-----------|
| memory (sesión 1→2) | ✅ existente: `vantadb-mcp/tests/demo_ai_ides.rs` + `ci-ai-ides-demo.yml` (claim F5, commit `ea60cc0c` previo F5.1 `f80ddadc`); se le añade comando local documentado en `AI_IDES.md` |
| verify (tamper → exit ≠0) | 🆕 `tests/wal_chain_verify.rs` (producer `#[ignore]`) + `scripts/demo-verify-e2e.ps1` + `wal-verify-demo.yml` (negativo: 2 clases de tamper) |
| verify (`delete --attest`/`certificate verify` → exit 0) | 🆕 mismo script, pasos [4/4] sobre store real vía CLI |
| governance (ACL → denied) | 🆕 producer `#[ignore]` en `ver04_governance.rs` + `scripts/demo-governance-e2e.ps1` + `injection-governance-demo.yml` |
| governance (bloque ≤ budget, `op:"injection"`) | 🆕 mismo demo: truncación visible en el wire (assert del test) + consulta Select-String del JSONL |
| comando + output esperado linkeado desde `docs/user/` | 🆕 §"Prove it" en `PRIVACY.md` (verify + governance) + comando local en `AI_IDES.md` (memory) |
| 0 `continue-on-error` | ninguno en los 2 workflows nuevos (verificado por `rg`) |

## Spec

> No es feature-add de producto (0 símbolos públicos nuevos: los producers son tests `#[ignore]`; scripts/workflows/docs no exponen contrato de librería). La tabla fija las decisiones de diseño del slice con evidencia.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Forma del demo verify (fixtures) | (A) producer test `#[ignore]` que siembra WALs + script CLI (patrón `icp02_privacy_demo.rs`) / (B) cirugía de bytes en PowerShell / (C) fixture binario commiteado | **A** — reusa `WalWriter` real (mismo formato v3), determinista, el script solo orquesta el binario real | ✅ decidido-por-evidencia: patrón F5 `icp02_privacy_demo.rs:204-546` + stop condition del plan (:1271 `cargo run -p vantadb --bin vanta-cli`) |
| 2 | Casos de tamper | (A) 1 caso (removed) / (B) 2 casos (removed + rewritten-con-CRC-recomputado) | **B** — el segundo es lo que CRC solo NO ve (claim central de VER-01); ambos deben dar exit ≠0 | ✅ decidido-por-evidencia: `src/wal.rs:1846-1909` + `WAL_INTEGRITY.md:15-17` |
| 3 | `delete --attest`/`certificate verify` dentro del demo verify | (A) incluirlo self-contained (put→attest→verify) / (B) declararlo cubierto por `icp02-privacy-demo.yml` y omitir | **A** — contrato: la cláusula verify los exige; auto-contenido; ⚠️ solape declarado con ICP-02 (allí es cadena PII; aquí es el roundtrip de la superficie) | ✅ decidido-por-evidencia: contrato verbatim + Pre-mortem F2 (declarar alcance por demo) |
| 4 | Forma del demo governance | (A) producer `#[ignore]` en `ver04_governance.rs` reusando helpers (una config: budget 60 + ACL `["persona/"]` → ok+truncated + denied en la MISMA corrida) / (B) test nuevo standalone | **A** — 1 request produce todas las filas del contrato; sin duplicar helpers | ✅ decidido-por-evidencia: `ver04_governance.rs:195-330` (c1/c2/c3) + `governance.rs:87-117` |
| 5 | Workflows | (A) 1 workflow por demo (`wal-verify-demo.yml`, `injection-governance-demo.yml`) / (B) 1 workflow con 2 jobs | **A** — triggers `paths` por superficie (verify=WAL/CLI; governance=proxy/memory); patrón F5 "workflow demo dedicado" | ✅ decidido-por-evidencia: `ci-ai-ides-demo.yml`/`icp02-privacy-demo.yml`/`ci-frameworks-demo.yml` |
| 6 | Consulta del audit en el script | (A) `Select-String` sobre el JSONL (patrón ICP-02 :97-105) / (B) `jq` | **A** — PowerShell en Windows+ubuntu sin instalar nada; precedente exacto | ✅ decidido-por-evidencia: `demo-privacy-e2e.ps1:96-105` |
| 7 | Docs | (A) §"Prove it" nueva en `PRIVACY.md` (verify+governance) + comando local en `AI_IDES.md` (memory) / (B) sólo task file | **A** — contrato: "linkeado desde `docs/user/`" | ✅ decidido-por-evidencia: `PRIVACY.md:114-145` (precedente) + `AI_IDES.md:70-73` |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) 0 cambios a comportamiento de producto (ningún archivo `src/**` de producto ni `vanta-proxy/src/**` se toca); (2) los 4 tests existentes de `wal_chain_verify` y los 4+1 de `ver04_governance` siguen verdes sin edición de su lógica; (3) los demos no requieren red ni tokens (offline determinista); (4) **NO tocar `evals/**`** (región co-batch VER-09 — solo lectura); (5) 0 `continue-on-error` en workflows nuevos.
- **Comandos de verificación:** `pwsh -NoProfile -File scripts/demo-verify-e2e.ps1` → PASSED · `pwsh -NoProfile -File scripts/demo-governance-e2e.ps1` → PASSED · `cargo nextest run -p vantadb --test wal_chain_verify --build-jobs 2` → 5/5 (4 + producer ignored skipped) · `cargo nextest run -p vanta-proxy --test ver04_governance` → 5/5 · `actionlint .github/workflows/wal-verify-demo.yml .github/workflows/injection-governance-demo.yml` → exit 0 · `node scripts/docs/check-links.mjs` + `check-docs.mjs` + `gen-index.mjs --check` → exit 0 · `scripts/validate-docs-coverage.ps1` → 0 gaps.
- **Deuda pendiente:** ninguna (slice aditivo). Nota: el demo verify solapa declaradamente el roundtrip certificado ya cubierto por `icp02-privacy-demo.yml` (decisión #3, alcance por demo declarado en el script/notas).

## Steps

### Step 1: Task file + discovery (este archivo)
- **Archivos:** `docs/dev/tasks/EXE-01.md`
- **Acción:** DISCOVERY completo: contrato verbatim, blast radius, Regla 0, Spec, steps. Gate: sin este archivo no hay edición (Regla 0).
- **Verify:** archivo existe con §Spec + §Impacto mapeado + §Steps; `## Spec` sin `N/A`.
- **Estado:** ✅ DONE (2026-09-30T01:46)

### Step 2: Demo verify (producer + script + workflow)
- **Archivos:** `tests/wal_chain_verify.rs` (refactor `seed_wal`→`seed_wal_into` + producer `#[ignore]` `wal_verify_demo_producer`) · `scripts/demo-verify-e2e.ps1` (nuevo) · `.github/workflows/wal-verify-demo.yml` (nuevo)
- **Acción:** fixtures `clean` + `tampered-removed` (extirpación) + `tampered-rewritten` (payload decodable alterado 'AAAA'→'BAAA' + CRC recomputado) + `handoff.json`; script: verify clean→exit 0/`ok:true`, tampered×2→exit ≠0/`ok:false` (negativo), roundtrip `put`→`delete --attest`→`certificate verify`→exit 0.
- **Verify:** `pwsh -NoProfile -File scripts/demo-verify-e2e.ps1` → PASSED (exit 0; los 2 tampered detectados) + `cargo nextest run -p vantadb --test wal_chain_verify --build-jobs 2` → 5/5 · `actionlint` workflow → exit 0.
- **Estado:** ✅ DONE (2026-09-30) — script **PASSED ×2**: `clean: exit 0, ok:true, statuses: verified` · `tampered_removed: exit 1, statuses: tampered` · `tampered_rewritten: exit 1, statuses: tampered` · `put + delete --attest + certificate verify: exit 0 x3` (cert `ok:true`, `residues_now:0`) · suite 4 passed + producer (ignored) ✅ · actionlint 0.

### Step 3: Demo governance (producer + script + workflow)
- **Archivos:** `vanta-proxy/tests/ver04_governance.rs` (producer `#[ignore]` `governance_demo_producer`) · `scripts/demo-governance-e2e.ps1` (nuevo) · `.github/workflows/injection-governance-demo.yml` (nuevo)
- **Acción:** producer: budget 60 + ACL `["persona/"]` + audit JSONL en `<demo>/`; asserts en wire (bloque ≤ budget + `…[truncated]`) y en audit (fila `ok` con `truncated=true`/`acl=allow`; fila `denied` con `acl=deny`; 0 payload). Script: consulta Select-String (op=injection / ok+truncated / denied+acl=deny) + control negativo de payload.
- **Verify:** `pwsh -NoProfile -File scripts/demo-governance-e2e.ps1` → PASSED + `cargo nextest run -p vanta-proxy --test ver04_governance` → 5/5 + `actionlint` → exit 0.
- **Estado:** ✅ DONE (2026-09-30) — script **PASSED ×2**: producer 1/1 (2 rows) · `audit: 2 injection rows (1 ok+truncated, 1 denied)` · `metadata-only: PERSONA-MARKER absent` · suite 4 passed + producer ✅ · actionlint 0. Filas reales: `persona/sess-1 ok …budget=60/60;truncated=true;acl=allow` · `scene/sess-1 denied …budget=60/60;acl=deny…`.

### Step 4: Docs (output documentado + linkeo desde docs/user/)
- **Archivos:** `docs/user/PRIVACY.md` (§"Prove it: WAL verification and injection governance": comando + output esperado + filas de audit + links a workflows) · `docs/user/AI_IDES.md` (comando local del demo memory + expected) · `docs/index.md` + `llms.txt` (regenerados)
- **Acción:** documentar cada demo con comando + output real + link; cargar `documentation-skill` gates.
- **Verify:** `node scripts/docs/check-links.mjs` → 0 rotos (exit 0) · `check-docs.mjs` → **GATING: all clear** · `gen-index.mjs --write` + `--check` → exit 0 · `scripts/validate-docs-coverage.ps1` → **0 gaps**.
- **Estado:** ✅ DONE (2026-09-30) — check-links exit 0 (45 broke preexistentes ≤ budget 58; ninguno nuevo) · check-docs all clear (tras añadir frontmatter al task file) · gen-index --check 0 · validate-docs-coverage 0 gaps.

### Step 5: Gates finales + cierre
- **Archivos:** (verificación, sin ediciones nuevas)
- **Acción:** gates mecánicos scoped: `cargo fmt --check` · clippy scoped (`-p vantadb --all-targets` y `-p vanta-proxy --all-targets`) · nextest de los 2 archivos tocados · `rg continue-on-error` en workflows nuevos = 0 · `git status` sin archivos fuera de scope (NO `evals/**`).
- **Verify:** todos ✅ + RESULTADO §7 (commit/self-review = LEAD por instrucción de wave).
- **Estado:** ✅ DONE (2026-09-30) — `cargo fmt --check` exit 0 · `cargo clippy -p vantadb --test wal_chain_verify -D warnings` exit 0 · `cargo clippy -p vanta-proxy --test ver04_governance -D warnings` exit 0 · `rg 'continue-on-error:'` workflows = 0 líneas (exit 1) · suites 4/4 + 4/4 · `git status` sin `evals/**` tocado (solo LEÍDO).

### Step 6: Micro-batch post-review (review: ✅ APPROVE — 2 Optional + 2 Nits)
- **Archivos:** `vanta-proxy/tests/ver04_governance.rs` (Opt 1) · `.github/workflows/{wal-verify-demo,injection-governance-demo}.yml` (Opt 2) · `docs/user/AI_IDES.md` (Nit 1)
- **Acción:** (Opt 1) assert autocontenido `!prefix.contains("how to deploy the service")` en el producer (la cláusula ACL ya no depende del test c2 que el workflow no corre) · (Opt 2) paths `Cargo.toml`+`Cargo.lock` en ambos workflows (paridad `ci-ai-ides-demo.yml`) · (Nit 1) output esperado literal del demo memory en `AI_IDES.md` · (Nit 2) **no aplicado** — drift preexistente `docs/dev/workflow/README.md:2,9` "28 active" vs 35 workflows reales; anotado acá.
- **Verify:** `pwsh scripts/demo-governance-e2e.ps1` → PASSED · `pwsh scripts/demo-verify-e2e.ps1` → PASSED · producers `--run-ignored ignored-only` → 1/1 ×2 · `actionlint` ambos → 0 · `markdownlint-cli2` (3 docs) → 0 · `cargo fmt --check` → 0 · `check-links`/`check-docs`/`gen-index --check` → 0/all-clear/0.
- **Estado:** ✅ DONE (2026-09-30T03:28) — evidencia: governance `audit: 2 injection rows (1 ok+truncated, 1 denied)` + `metadata-only` (con el assert nuevo) · verify `clean 0 / removed 1 / rewritten 1 / roundtrip 0×3` · producers `PASS (1/1)` c/u.

## Dependencias
- Superficies ✅: VER-01 (`0cc14247`), VER-02 (`ccc51d09`), VER-04 (`575ce8dd`) — shipped F4.
- Patrón demos F5 ✅: ICP-01 `f80ddadc` (memory demo), ICP-02 `48fb88d0` (script E2E), ICP-03 `ea60cc0c`.
- Co-batch: VER-09 (wave F6) — región `evals/**` PROHIBIDA (solo lectura).
- nextTask (cierre de campaña): N-17 (Task 49) / cierre F6 = lane LEAD.

## Review (GATE — agente distinto, P2-01)

> Review ejecutado por contexto distinto (vía LEAD), conforme a `10. CIERRE: NO commit · NO self-review`.

- **Revisor:** review fresco vía LEAD (2026-09-30) — tier Fast (paths CI/docs/tests; ningún path adversarial de wire).
- **Enfoque:** ¿los demos prueban la garantía (negativos incluidos) o son decorativos?
- **Cómo se probó:** salidas reales de los scripts (capturadas en §Notas) + nextest scoped + actionlint + gates docs.
- **Veredicto:** ✅ **APPROVE** (0 Critical/Required; 2 Optional + 2 Nits). Micro-batch: Opt 1 ✅ (assert ACL autocontenido en producer) · Opt 2 ✅ (`Cargo.toml`/`Cargo.lock` en paths de ambos workflows) · Nit 1 ✅ (output literal nextest del demo memory en `AI_IDES.md`) · Nit 2 ⚠️ no aplicado (drift preexistente `workflow/README.md`, anotado en §Notas).

## Notas

- **Alcance (re-baseline plan :1262):** memory ✅ ya existía; el slice cierra verify (tamper WAL, hoy solo test) + governance (budget/ACL/audit, hoy solo test) con output documentado. Solape con ICP-02 declarado: allí `delete --attest`/`certificate verify` corren dentro de la cadena PII; el demo verify los corre en el roundtrip de la superficie (self-contained) y añade el tamper del WAL, que no tenía demo en CI (`rg "vanta-cli verify" .github/workflows` = 0 antes de este cambio).
- **Decisión cero-código-producto:** ningún cambio a `src/**` de producto ni `vanta-proxy/src/**`; solo tests `#[ignore]`, scripts, workflows, docs.
- **Salidas reales de los scripts (2026-09-30, corrida ×2 c/u):**
  - verify: `clean: exit 0, ok:true, statuses: verified` · `tampered_removed: exit 1, statuses: tampered` · `tampered_rewritten: exit 1, statuses: tampered` · `put + delete --attest + certificate verify: exit 0 x3` (certificate `ok:true`, `residues_now:0`).
  - governance: `audit: 2 injection rows (1 ok+truncated, 1 denied)` · `metadata-only: PERSONA-MARKER absent from the audit`; filas: `persona/sess-1 ok …budget=60/60;truncated=true;acl=allow`, `scene/sess-1 denied …budget=60/60;acl=deny…`.
- **Incidente local (no bloqueante):** disco C: a 0 bytes durante el link del test de governance → se liberó `target/debug/incremental` + artefactos regenerables (`target/{icp02-demo,package,doc,criterion}`) y se corrió con `CARGO_INCREMENTAL=0`; verificación completa después. No afecta CI.
- **Índices generados (scope co-batch):** `docs/index.md` + `llms.txt` incluyen además la entrada de `VER-09` (task file del worker co-batch) + catch-up de `NOTION-SYNC` (commit 920353b9, description ya en HEAD). La composición del commit/PR es decisión del LEAD (precedente wave: changesets entrelazados). `docs/dev/tasks/VER-09.md` y `evals/runners/` son WIP ajeno — **no tocados**.
- **Gate P/D:** no dispara (no feature-add; sin símbolos públicos nuevos).
- **SECURITY:** no aplica (sin trust boundaries nuevos; fixtures sintéticas, offline, sin red/keys; audit metadata-only verificado por assert + consulta).
- **PERFORMANCE:** no aplica (sin hot paths; tests `#[ignore]` no corren por default).
- **Ponytail:** se reusó el patrón F5 (`icp02_privacy_demo.rs` producer+script+workflow) y los helpers existentes de ambos test files; 0 abstracciones nuevas.
- **Micro-batch post-review (2026-09-30T03:28):** Opt 1 — el producer governance ahora asserta también en el wire que el contenido del scene denegado (`how to deploy the service`) nunca entra al prompt (autocontenido: el workflow corre solo este test; antes la cláusula ACL wire-level vivía en c2, no ejecutado por el workflow). Opt 2 — `Cargo.toml`/`Cargo.lock` añadidos a los `paths` de ambos workflows (paridad `ci-ai-ides-demo.yml`). Nit 1 — output esperado literal del demo memory en `AI_IDES.md` (capturado de corrida real: `Summary [   1.415s] 1 test run: 1 passed, 0 skipped`). Nit 2 — **no aplicado**: `docs/dev/workflow/README.md:2,9` declara "28 active" con 35 workflows reales en `.github/workflows/` (drift preexistente, fuera de scope de EXE-01; anotado para triage del LEAD). Re-verify: ambos scripts PASSED, producers 1/1 ×2, actionlint 0 ×2, markdownlint 0, fmt 0, gates docs verdes.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — patrón F5 verificado; superficies shipped; decisiones #1-7 resueltas por evidencia |
| Pendientes de ejecución (downhill) | 0 (S1–S6 ✅; commit = LEAD) |
| % completado | 100% (implementación + verificación local + micro-batch post-review; cierre formal LEAD) |
