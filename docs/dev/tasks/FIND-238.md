---
title: "FIND-238: WASM/npm — silenciar logs DEBUG de Client.create() (tracing-wasm default-on sin filtro)"
kind: task
description: "El init() del binding WASM instala tracing-wasm con max_level TRACE (default del builder) y los debug! de src/config.rs inundan el console al crear el cliente; el fix fija WARN por defecto con gate globalThis.VANTADB_LOG (análogo portable de RUST_LOG) documentado en el README del paquete."
---

# FIND-238: WASM/npm: silenciar logs DEBUG de `Client.create()` (`tracing-wasm` default-on sin filtro)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 2, F0)
- **Fuente:** validación externa v0.8.0 (2026-10-03) → fila FIND-238 (Backlog DELTA)
- **Esfuerzo:** 🟢 2-3h
- **Prioridad:** 🟡
- **Tipo:** Mixto (Rust WASM + smoke TS + docs)
- **Turns estimados:** 8-12
- **Creado:** 2026-10-04T02:10
- **last-synced:** 2026-10-04T02:10
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 4 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `init()` — 6 call sites, todos en `vantadb-wasm/src/lib.rs`: `new` (:488), `open` (:510), `connect_persistent` (:539), `connect_idb` (:572), `connect_worker` (:616, feature `opfs`) + definición (:1935). CodeGraph: `TRACING_INIT` = 1 caller (`init`), 0 callers externos al crate |
| Callees | `tracing_wasm::{set_as_global_default_with_config, WASMLayerConfigBuilder}` (0.2.1, ya en el árbol vía Cargo.lock), `tracing::Level` (nuevo dep directo — crate ya transitivo vía core/tracing-wasm), `console_error_panic_hook::set_once` (intacto), `js_sys::Reflect::get/global` (ya usado en el crate) |
| Implicaciones | Sin cambio de API pública ni funcional: mismos métodos/firmas; solo cambia el nivel del subscriber de consola instalado en `init()`. `--no-default-features` sigue compilando (sin `tracing-wasm` → sin subscriber; panic hook sigue). Cargo.lock suma `tracing` a las deps de `vantadb-wasm` (crate ya presente → cero costo de árbol). Bundle wasm: sin cambio apreciable (el subscriber ya estaba). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-wasm/Cargo.toml` (56L), `vantadb-wasm/src/lib.rs` (init + call sites :470-662, :1900-1941, mod tests :2169-2233 vía codegraph/read), `src/config.rs:980-1049` (origen de los `debug!`), `src/console.rs` (348L — patrón `RUST_LOG`), `vantadb-wasm/README.md` (§3 features L140-209), `vantadb-ts/vitest.config.ts`, `vantadb-ts/package.json`, `.github/workflows/release-npm-61.yml` (jobs `tests`/`publish-wasm`), fuente local `tracing-wasm-0.2.1` (`~/.cargo/registry`, lib.rs completo del crate), `tracing-core-0.1.35` (`metadata.rs` FromStr de `Level`), `.opencode/rules/js-ecosystem.md`, `clean-code-clean-architecture.md` Apéndice V.
- **Archivos referenciados hacia dentro (imports/deps de los editados):** `vantadb-wasm/src/lib.rs` ← `tracing_wasm` (cfg feature), `console_error_panic_hook`, `js_sys`; `Cargo.toml` ← versiones del workspace (`version.workspace`, `[lints] workspace`).
- **Archivos que referencian a los editados (referencias entrantes):** grep `tracing-wasm` = `Cargo.toml` (feature) + `lib.rs` (`#[cfg]`) + `README.md` (§3) + workflows (`wasm-pack build`) + `vantadb-ts` (consume `pkg/` vía `file:../vantadb-wasm/pkg`). `init()` no se llama fuera de `lib.rs` (codegraph). Sin consumidores del nivel de log (nadie lo configura hoy).
- **Veredicto impacto:** bajo — cambio localizado al `init()` del binding + declaración de feature + docs del paquete; la suite vitest de `vantadb-ts` (278+ tests) cubre regresión funcional del pkg.

## Contrato

"`Client.create()` con el wasm build NO emite líneas `DEBUG` por defecto (nivel default WARN) **y** `globalThis.VANTADB_LOG` documentado permite re-activar (`trace|debug|info|warn|error`); smoke del paquete (`vantadb-ts`) sin DEBUG; sin cambio funcional."

Verificación exacta:
1. RED→GREEN: `cd vantadb-ts && npx vitest run src/__tests__/find238.test.ts` (falla contra pkg buggy; verde post-fix)
2. Build wasm local: `cd vantadb-wasm && wasm-pack build --release` + `node dev-tools/build-wasm-types.mjs` (desde raíz)
3. Suite TS completa: `cd vantadb-ts && npm test` (verde)
4. `cargo fmt --check` + `cargo check -p vantadb-wasm --target wasm32-unknown-unknown`

## Spec (decisiones resueltas por plan + evidencia de código)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Mecanismo | A: `set_as_global_default_with_config(WASMLayerConfigBuilder::set_max_level)` vs B: sacar `tracing-wasm` de default (opt-in, stop-condition) vs C: silenciar en core | A | ✅ Evidencia: el builder expone filtro real (`WASMLayer::enabled()` = `level <= max_level`, fuente tracing-wasm-0.2.1) → la stop-condition (B) no aplica |
| 2 | Nivel default | A: WARN vs B: INFO | A | ✅ Plan contrato "WARN/INFO"; WARN = primera impresión limpia (safe default — incremental-implementation Rule 4) |
| 3 | Gate de escape | A: `globalThis.VANTADB_LOG` (string, portable) vs B: sin gate (deja "logs útiles ocultos" sin mitigación) vs C: `RUST_LOG` solo-Node | A | ✅ Plan pre-mortem(1): "gate por env documentado"; B contradice el Risk Register; C no funciona en browser |
| 4 | Ubicación del gate | A: leído en `init()` (primer cliente gana) vs B: re-init por cliente | A | ✅ `set_global_default` es one-shot por proceso (tracing); B requeriría registry propio/unsafe |
| 5 | Dep directa | A: `tracing = { version = "0.1", optional = true }` + feature explícita vs B: no gate | A | ✅ `Level` es el tipo de `tracing`; el crate ya está en el árbol (0.1.44 vía core + tracing-wasm) → cero costo de compilación/tamaño |
| 6 | ¿Gate D (question)? | A: no dispara — fix especificado por plan aprobado, sin símbolos públicos nuevos (no `pub fn`/endpoint/método de binding) vs B: preguntar | A | ✅ El plan F0 full-detail ya decidió approach + pre-mortem; `VANTADB_LOG` es configuración de logging, no símbolo de API |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. Superficie pública del binding intacta: mismos métodos/firmas (`Client::new/open/connect_*`) — sin cambio funcional.
  2. `wasm-pack build --no-default-features` sigue compilando (sin `tracing-wasm` → sin subscriber; `console_error_panic_hook` sigue activo).
  3. `VANTADB_OPENAI_API_KEY` nunca se loguea por valor (solo `present = false`, `src/config.rs:1042-1047`) — el fix no altera ese comportamiento.
  4. El gate se lee UNA vez por proceso (primer cliente); documentado así en el README.
- **Comandos de verificación:** `cd vantadb-ts && npx vitest run src/__tests__/find238.test.ts` (verde) · `cd vantadb-ts && npm test` (suite completa verde) · `cd vantadb-wasm && wasm-pack build --release` (compila).
- **Deuda pendiente:** ninguna.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda. Reduce ruido de salida (mejora DX de la superficie npm) sin atajos nuevos; el gate reusa el patrón `RUST_LOG` del repo (`src/console.rs:109-117`).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato ✅ (sin DEBUG por defecto + gate documentado + smoke verde) + fmt/clippy/build wasm del área |
| **Commit** | Atómico, `fix(wasm): FIND-238 — ...`, verificación mecánica registrada |
| **Release** | Entrada de changelog (release-plz, patch) — n/a en este commit; queda para 0.9.0 |

## Herramientas necesarias
- Terminal (wasm-pack, cargo check/fmt, vitest) — MCPs de Rust deshabilitados por default
- `codegraph_codegraph_explore` (blast radius de `init()`), `codebase-memory-mcp_check_index_coverage` (`vantadb-wasm/src/lib.rs`, `Cargo.toml`)
- `campaign_verify_cmd` (contrato), `campaign_update_task_state` (recitation)
- `pwsh dev-tools/ocr-review.ps1` (cierre)

**Skills cargadas (SDP):** `campaign-executor` · `progreso` · `ponytail` (base, auto) · `source-driven-development` (API `tracing-wasm 0.2.1` validada en docs.rs + fuente del crate) · `test-driven-development` (Prove-It: smoke RED antes del fix) · `incremental-implementation` (slice único) · `context-engineering` · `doubt-driven-development` · `security-and-hardening` (pin SDP v3: trust boundary — logging) · `ci-cd-and-automation` (job TS de `release-npm-61`) · `documentation-skill` (README del paquete + task file) · `coordinated-web-search` (validación contra docs oficiales).

## Investigation Notes
- **Causa raíz (Phase 1):** `vantadb-wasm/Cargo.toml:42` (`default = ["tracing-wasm"]`) + `lib.rs:1939` (`set_as_global_default()` sin nivel) → builder default `max_level = tracing::Level::TRACE` (fuente tracing-wasm-0.2.1 `lib.rs:191`) → los `debug!` de `Config::default()` (`src/config.rs:1004+`, ~40 líneas por open) salen al console sin filtro.
- **Repro (RED, contra pkg actual):** `cd vantadb-ts && node --input-type=module -e "import('vantadb-wasm').then(m => new m.Client())"` → 40+ líneas `DEBUG src\config.rs:...` (evidencia capturada 2026-10-04, pkg pre-fix).
- **API verificada (Phase 3):** docs.rs + fuente local `tracing-wasm-0.2.1`: `WASMLayerConfigBuilder::new().set_max_level(tracing::Level).build()` + `set_as_global_default_with_config(config)`; `WASMLayer::enabled()` filtra `level <= max_level` (filtro real, no cosmético). `tracing::Level: FromStr` case-insensitive (`tracing-core-0.1.35 metadata.rs:553`). **NO existe** `set_as_global_default_with_level` (el pre-mortem del plan lo conjeturaba; la API real es `_with_config`).
- **Patrón repo:** el core usa `RUST_LOG` con default `info` (`src/console.rs:109-117`); WASM no tiene env → `globalThis.VANTADB_LOG` como análogo portable (Node + browser).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — API del crate y mecanismo verificados contra fuente |
| Pendientes de ejecución (downhill) | 1 — step 4 (cierre) |
| % completado | 75% |

## Fase 1 — Evidencia de Debugging (GATE — bug)

- **Repro:** `cd vantadb-ts && node --input-type=module -e "import('vantadb-wasm').then(m => { const c = new m.Client(); c.close(); })"` → 40+ líneas `DEBUG src\config.rs:<n> VANTADB_*` (pkg pre-fix; validación externa v0.8.0 lo reportó como "logs verbosos en Node al crear el cliente").
- **Hipótesis:** el subscriber de `tracing-wasm` se instala con el default del builder (`TRACE`) y el binding nunca pasa un `max_level`; los `debug!` del core pasan el filtro (`level <= max_level`).
- **1 variable controlada:** el `max_level` del subscriber instalado en `init()` (default TRACE → WARN + gate). Nada más: sin tocar core, sin tocar la API del binding.
- **Test RED:** `vantadb-ts/src/__tests__/find238.test.ts` (smoke en proceso hijo Node: (a) sin DEBUG por defecto, (b) con `globalThis.VANTADB_LOG="debug"` sí hay DEBUG) — se ejecuta ANTES del fix contra el pkg buggy → FALLA por (a); post-fix → VERDE.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY — aplica (pin SDP v3: trust boundary/logging):** el cambio **reduce** superficie de información — los `debug!` exponían la config/env resuelta (`VANTADB_STORAGE_PATH`, modelos, etc.) al console por defecto en cada `Client.create()`. Se preserva que `VANTADB_OPENAI_API_KEY` nunca se loguea por valor. El gate parsea un string de `globalThis` con fallback estricto a `WARN` (sin panic, sin eval, sin default permisivo). `console_error_panic_hook` intacto. Sin dependencias nuevas en el árbol (`tracing` ya transitivo). Hallazgo: sin Critical/High.
- [x] **PERFORMANCE — no aplica:** `init()` corre 1 vez por proceso; leer un global + parsear un string es O(1) en startup. El subscriber ya existía; no toca hot paths (search/ingest/serialización).

## Steps

### Step 1: RED — smoke test en `vantadb-ts` (contrato de logs)
- **Archivos:** `vantadb-ts/src/__tests__/find238.test.ts` (nuevo)
- **Acción:** test vitest que spawnea un proceso Node hijo por caso: (a) `new Client()` sin gate → sin `DEBUG` en stdout+stderr + creación OK; (b) `globalThis.VANTADB_LOG="debug"` → `DEBUG` presente. Ejecutar contra el pkg pre-fix.
- **Verify:** `cd vantadb-ts && npx vitest run src/__tests__/find238.test.ts` → FALLA (RED) por (a) con las líneas DEBUG reales.
- **Estado:** ✅ COMPLETED — RED verificado 2026-10-04T02:12: `1 failed | 1 passed`; el fallo es (a) `not.toMatch(/DEBUG/)` con 40+ líneas `DEBUG src\config.rs:*` reales en stdout.

### Step 2: GREEN — nivel WARN + gate `VANTADB_LOG` + README
- **Archivos:** `vantadb-wasm/Cargo.toml`, `vantadb-wasm/src/lib.rs` (`init()` :1935-1941), `vantadb-wasm/README.md`, `vantadb-ts/README.md`
- **Acción:** `tracing = { version = "0.1", optional = true }` + feature explícita `tracing-wasm = ["dep:tracing-wasm", "dep:tracing"]`; `init()` usa `set_as_global_default_with_config(builder.set_max_level(console_log_level()))` con helper cfg'd que lee `globalThis.VANTADB_LOG` (fallback `WARN`); READMEs documentan el gate.
- **Verify:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` ✅ + rebuild pkg (`wasm-pack build --release` 3m26s + `node dev-tools/build-wasm-types.mjs`) + test Step 1 → VERDE.
- **Estado:** ✅ COMPLETED — GREEN verificado 2026-10-04T02:19: build wasm ✅ + `2 passed (2)`.

### Step 3: VERIFY full del área
- **Archivos:** —
- **Acción:** fmt + check wasm + suite TS completa (regresión funcional del pkg) vía `campaign_verify_cmd`.
- **Verify:** `cargo fmt --check` ✅ · `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` ✅ · `cd vantadb-ts && npm test` ✅ (278+ tests) · `npx tsc --noEmit` (vantadb-ts) ✅.
- **Estado:** ✅ COMPLETED — evidencia 2026-10-04T02:20: `npm test` 17 files / **334 passed** · `tsc --noEmit` exit 0 · `eslint` exit 0 · `cargo fmt -p vantadb-wasm --check` exit 0 · `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` ✅ · `--no-default-features` ✅ (invariante 2). `cargo fmt --check` workspace falla por `src/bin/vanta-cli.rs` (trabajo FIND-237 en vuelo, ajeno) y `cargo clippy -p vantadb-wasm --target wasm32 -- -D warnings` falla por `src/index/serialize/file.rs:146` (core, pre-existente, no gateado) — ambos documentados en Notas.

### Step 4: CIERRE — OCR + review P2-01 + commit local + recitation
- **Archivos:** `docs/dev/tasks/FIND-238.md`
- **Acción:** `pwsh dev-tools/ocr-review.ps1` (Critical/High bloquean); fork de review a `vanta-review` (agente distinto); commit local `fix(wasm): FIND-238 — ...`; `campaign_update_task_state` + `skill progreso`.
- **Verify:** OCR sin Critical/High + veredicto review registrado + `git status` limpio de archivos ajenos (`opencode.jsonc` excluido).
- **Estado:** ✅ COMPLETED — OCR spec 0 findings propios; review `vanta-review` ✅ approve (sesión `ses_efa647686ffedhX5WYhVg2iOoR`); commit local (hash en la recitation MCP).

## Dependencias
- F0 wave 0 — sin dependencias. nextTask: FIND-239.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (leaf, contexto fresco — sesión `ses_efa647686ffedhX5WYhVg2iOoR`, distinta al implementador). Fast tier (paths: `vantadb-wasm/**`, `vantadb-ts/**`, `docs/dev/tasks/**`, `Cargo.lock` — ningún glob adversarial). OCR delegation ejecutado antes del veredicto: 2 archivos propios revisables vs sus Rule Groups (Cargo manifest hygiene + Rust genérico + js-ecosystem) → 0 Critical/High/Medium.
- **Enfoque:** approach vs alternativas (default WARN + gate vs opt-in `tracing-wasm` vs silenciar en core); discriminación del smoke (falsos verdes); edge cases del gate (valor no-string/inválido); `--no-default-features`; docs vs comportamiento real.
- **Cómo se probó:** el revisor re-ejecutó el comando del contrato (`npx vitest run src/__tests__/find238.test.ts` → 2/2 passed contra el pkg reconstruido, mtime 02:19), corrió sus propios probes de borde (`VANTADB_LOG=123` y `"bogus"` → fallback WARN sin crash, sin DEBUG) y re-verificó fmt/no-default-features/Cargo.lock; confirmó el filtro real en la fuente local del crate (`enabled()` = `level <= max_level`).
- **Checklist anti-hábitos tóxicos:** sin salidas inventadas (cada claim con comando/archivo) · sin done sin verificar (contrato re-corrido por el revisor) · fallos reportados (RED citado como evidencia histórica no re-ejecutable sin pkg viejo — declarado por el revisor) · sin scope creep · cobertura SDP completa.
- **Veredicto:** ✅ **approve** — hallazgos Low: (1) [proceso] `vantadb-ts/README.md` quedó commiteado en `2ef38561` (barrido por FIND-239; el commit propio NO debe incluirlo); (2) [test] el smoke prueba `new Client()` wasm (mismo path `init()` que `Client.create()` del wrapper TS — solo drift de redacción); (3) [test] el caso (b) depende de que `config.rs` siga emitiendo `debug!` (canario aceptable).

## Notas
- Pre-mortem del plan cubierto: (1) ocultar señal → gate `VANTADB_LOG` documentado en README; (2) API del crate → validada contra fuente real (era `_with_config`, no `_with_level`); (3) build wasm roto → `wasm-pack build --release` local en Step 2/3 (✅ 3m26s).
- **Hallazgo colateral (pre-existente, NO tocado):** `cargo clippy -p vantadb-wasm --target wasm32-unknown-unknown -- -D warnings` falla en `src/index/serialize/file.rs:146` (`clippy::drop_non_drop`, rust-1.95) — core, fuera del blast radius de FIND-238 y de su owner (Engine/Arch); la configuración no está gateada en CI (CI usa `wasm-pack build`/`wasm-pack test`, no clippy wasm32). Candidato a fila FIND derivada — decisión del orquestador.
- **Hallazgo colateral (ajeno, en vuelo):** `cargo fmt --check` workspace falla por `src/bin/vanta-cli.rs` (cambios sin commitear de FIND-237, otro worker); mis archivos verifican con `cargo fmt -p vantadb-wasm --check` (exit 0).
- `NOTICED BUT NOT TOUCHING:` el builder default también trae `report_logs_in_timings: true` (performance marks en devtools) — fuera del contrato de FIND-238; `console_warn` ad-hoc (`lib.rs:470`) es un path separado, sin cambios.
- Cargo.lock: `tracing` agregado a las deps de `vantadb-wasm` (crate ya presente en el árbol vía core + tracing-wasm; sin nodos nuevos).
- **Contaminación cruzada de working tree (documentada):** durante la sesión, el worker de FIND-239 commiteó `2ef38561` y barrió mi edición de `vantadb-ts/README.md` (sección Console logging) — el contenido quedó correcto en HEAD y mi commit no lo re-incluye (archivo ya limpio). Nada más propio fue barrido (`Cargo.toml`/`lib.rs`/`README` wasm siguen ` M` al momento del cierre).

## Context Save Point

- **Última acción:** Steps 1-3 ✅ (RED→GREEN + verify full: suite TS 334/334, check wasm32, no-default-features, fmt crate). Pendiente cierre.
- **Próximo step:** Step 4 (OCR + review P2-01 + commit local).
- **Archivos en vuelo:** `vantadb-wasm/Cargo.toml`, `vantadb-wasm/src/lib.rs`, `vantadb-wasm/README.md`, `vantadb-ts/README.md`, `vantadb-ts/src/__tests__/find238.test.ts`, `Cargo.lock`.

## RESULTADO §7

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: 805a7668 (fix(wasm): FIND-238 — silenciar logs DEBUG de Client.create())
ARCHIVOS: vantadb-wasm/Cargo.toml, vantadb-wasm/src/lib.rs, vantadb-wasm/README.md, vantadb-ts/src/__tests__/find238.test.ts, Cargo.lock, docs/dev/tasks/FIND-238.md · (vantadb-ts/README.md quedó en 2ef38561 vía FIND-239)
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:no | fix plan-aprobado; sin símbolos nuevos; sin ambigüedad
SKILLS_CARGADAS: source-driven-development, test-driven-development, incremental-implementation, context-engineering, doubt-driven-development, security-and-hardening (pinned), ci-cd-and-automation, documentation-skill, coordinated-web-search + base (campaign-executor, progreso, ponytail)
```
