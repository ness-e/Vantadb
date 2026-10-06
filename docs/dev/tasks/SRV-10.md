---
title: "TASK SRV-10: Cifrado en reposo del server (HTTP) — re-scope (b): superficie honesta"
kind: task
description: "Re-scope obligatorio (FIND-249 diferido): superficie de config/key mgmt del server + aviso al arrancar cuando VANTADB_ENCRYPTION_KEY se setea sin la feature `encryption` compilada + docs honestas (sin claim de protección) + tests. Feature opt-in `encryption` en vantadb-server. Sin wiring on-disk (FIND-249)."
---

# TASK SRV-10: Cifrado en reposo del server (HTTP) — re-scope (b): superficie honesta

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 70, F5 — última de la fase)
- **Fuente:** plan Task 70 + FIND-249 (`docs/dev/Backlog.md:384`) + ENC-01 (`b02b1609`) + DUR-02
- **Esfuerzo:** 🟡 3-5d (plan) → ejecución real del contrato reducido ~4-6h
- **Appetite:** 1sem
- **Prioridad:** 🟡
- **Tipo:** Mixto (Rust server + Docs)
- **Turns estimados:** 12-18
- **Creado:** 2026-10-06T06:08 | **last-synced:** 2026-10-06T06:48
- **Estado:** ⏳ IN PROGRESS (Steps 1-3 ✅; cierre: review P2-01 + commit)
- **Incógnitas (uphill):** 0 — re-scope resuelto en DISCOVERY con evidencia (§Re-scope)
- **Pendientes (downhill):** 1 step (Step 4 — cierre)
- **Campaign ID:** master-plan-0.9.0-20261004

## Re-scope (Gate Result — decisión de DISCOVERY, con evidencia)

**Decisión: (b) contrato reducido.** El wiring on-disk está diferido como FIND-249 por decisión del owner (2026-10-04: "avisar ahora, cablear luego"); el contrato pleno ("server con cifrado en reposo activable end-to-end") es inalcanzable hoy sin duplicar FIND-249 — stop condition explícita: "no inventes cifrado parcial".

Evidencia:

1. **FIND-249 sigue diferido:** `Backlog.md:384` — estado `🆕 Pendiente`; "Decisión owner 2026-10-04: avisar ahora (ENC-01, plan) + cableado completo DIFERIDO (post-fases)". `with_cipher` = 0 callers; 0/6 artefactos on-disk cifrados (DUR-02, verificado HEAD 2026-10-04).
2. **ENC-01 cerrado** (`b02b1609`): aviso honesto en `StorageEngine::open_with_config` (`src/storage/engine/init.rs:614-643`), gated por feature `encryption` + key + backend on-disk.
3. **Gap real de la superficie server (verificado 2026-10-06):**
   - `vantadb-server` NO compila `encryption` — RED mecánico: `cargo check -p vantadb-server --features encryption` → `error: the package 'vantadb-server' does not contain this feature: encryption`.
   - `Config::from_env()` (= `Config::default()`, `src/config.rs:1484-1486`) lee `VANTADB_ENCRYPTION_KEY` incondicionalmente (`src/config.rs:1421-1427`) → la key queda en config pero **ningún consumidor la usa** en el proceso server (grep `encryption_key` en `*.rs`: solo `crypto.rs:198` [feature-gated] e `init.rs:621` [feature-gated]).
   - Con la feature apagada (build default del server) el aviso ENC-01 **nunca** dispara → la key es **silenciosamente ignorada**: exactamente la "falsa sensación de seguridad" del pre-mortem #3, en la superficie donde el operador configura la key.
4. **Pre-mortem #3 verificado:** la ruta server SÍ pasa por `StorageEngine::open_with_config` (`src/server/bootstrap.rs:314` → `init.rs`) → el aviso ENC-01 **cubre la ruta server cuando la feature está compilada**. Mitigación completa = habilitar la feature opt-in (forwarding) + avisar en el caso no-compilado.

**Frontera declarada (anti-duplicación FIND-249):** SRV-10 **NO toca** write paths de storage, WAL, HNSW, VantaFile, backend-KV, text_index, snapshots, `src/crypto.rs`, `src/storage/vfile.rs`, ni introduce callers de `with_cipher`. SRV-10 = superficie server: feature opt-in + aviso key-sin-feature + docs + tests. El cifrado real on-disk queda en **FIND-249** (diferido, 3-5d).

**Fuera de alcance declarado:** ruta MCP (`vantadb-server --mcp` → `run_stdio_server_auto`, crate `vantadb-mcp`) — superficie distinta (SRV-10 = "server (HTTP)"); la nota ENC-01 en FIND-249 ya difiere el "counterpart core-wide" al cableado.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `run_with_hooks` (`src/server/bootstrap.rs:302`) — 3 callers: `bootstrap::run` (:287), `server::mod`/`server::routing` (facades). Cadena real: `vanta-cli server --http` (`src/cli_handlers/server.rs:310` → `cli_server::run`) y `vantadb-server` (`vantadb-server/src/main.rs:92`). Aviso nuevo = log a stderr en el arranque; sin cambio de flujo. |
| Callees | bootstrap.rs → `crate::config::Config` (`encryption_key`, `backend_kind`), `crate::storage::BackendKind` (import local cfg'd), `crate::console::warn` (ya usado por `validate_auth_config`). `vantadb-server/Cargo.toml` → feature nueva `encryption = ["vantadb/encryption"]`. Sin dependencias nuevas. |
| Implicaciones | **Contrato público:** feature cargo opt-in de `vantadb-server` (crate `publish = false`); sin símbolos Rust nuevos (fns privadas cfg'd); `tests/api/public-api.txt` (HARD-01) intacto. **Comportamiento:** +1 warning stderr al arrancar el HTTP server SOLO si key set + feature no compilada; sin key o con feature → byte-idéntico. **Performance:** nula (comparación de `Option` + 1 log por arranque; no hot path). **Migración:** ninguna. **Tests:** 3 unit nuevos feature-condicionados en `bootstrap.rs`; suites existentes intactas. **Docs:** `HTTP_API.md:772-774`, `CONFIGURATION.md:74,499`, `hardening.md:280` (nota en `Backlog.md` OMITIDA — edición concurrente de WIRE-13). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `src/server/bootstrap.rs` (752L) — `validate_auth_config` (:41-89), `log_security_mode` (:91-125), `run_with_hooks` (:302-422), `#[cfg(test)] mod tests` (final del archivo). Punto de inserción: call tras `validate_auth_config(&config)?` (:310); helpers tras `log_security_mode` (:125); tests dentro del `mod tests` existente.
  - `vantadb-server/Cargo.toml` (88L) — deps `vantadb` con feature list explícita (:16-26); `[features]` (:62-68, sin `encryption`); precedente `tls = ["vantadb/tls"]`.
  - `vantadb-server/src/main.rs` (166L) — `Config::from_env()` (:52) → `run_with_hooks` (:92); dispatch MCP (:54-67).
  - `src/storage/engine/init.rs` (sección ENC-01 :614-643) — notice + gate (feature+key+on-disk).
  - `src/storage/engine/tests/init.rs` (:795-905) — patrón ENC-01: pinneo explícito de config (hermeticidad), exclusión InMemory.
  - `src/config.rs` (secciones funcionales): campo `encryption_key` (:888-895), carga env (:1421-1427), builder `with_encryption` (:1704-1712), `from_env` (:1484-1486), Default backend (:1158).
  - `docs/api/HTTP_API.md` (:740-793) — bullet "No encryption at rest" (:772-774) en "Where VantaDB is honestly behind".
  - `docs/user/operations/CONFIGURATION.md` — fila `encryption_key` (:74), §7 fila `encryption` (:499).
  - `docs/user/operations/hardening.md` (:275-284) — "Next Steps": línea SRV-10 (:280).
- **Archivos referenciados hacia dentro (imports/dependencias):** bootstrap.rs → `crate::{config::Config, storage::StorageEngine, console, server::telemetry}`; Cargo.toml server → `vantadb` (path, features explícitas); main.rs → `vantadb::cli_server`.
- **Referencias entrantes (CodeGraph/grep):** `run_with_hooks` — 3 callers (bootstrap/mod/routing) + hosts externos vía facades; `encryption_key` — consumidores `.rs`: `crypto.rs:198` (feature-gated), `init.rs:621` (feature-gated), `config.rs` (definición/carga); feature set de `vantadb-server` — consumido por `--all-features` de CI/verify y por humanos (docs). `BackendKind` re-exportado en `src/storage/mod.rs:12` (`crate::storage::BackendKind`).
- **Veredicto impacto:** **BAJO** — log de arranque + feature opt-in aditiva; sin símbolos públicos, sin cambios on-disk, sin hot path, sin migración. Riesgos del pre-mortem mitigados: (1) re-scope documentado (b) con evidencia mecánica; (2) frontera anti-FIND-249 declarada; (3) aviso ENC-01 verificado en la ruta server (`bootstrap.rs:314`) + aviso nuevo para el caso no-compilado.

## Contrato (reducido — re-scope b)

"Con la feature `encryption` NO compilada (build default de `vantadb-server`/`vanta-cli`), setear `VANTADB_ENCRYPTION_KEY` hace que el HTTP server emita al arrancar un warning explícito de que la key es **IGNORADA** y los datos on-disk quedan **PLAINTEXT** (referencia FIND-249; guía a OS/volume-level encryption); sin key o con la feature compilada el arranque es byte-idéntico (los negativos lo fijan); `vantadb-server` gana la feature opt-in `encryption = ["vantadb/encryption"]` (`cargo check -p vantadb-server --features encryption` verde → con ella el aviso ENC-01 cubre la ruta server vía `open_with_config`); `HTTP_API.md`/`CONFIGURATION.md`/`hardening.md` actualizados sin prometer protección; tests feature-condicionados fijan aviso positivo + negativos (sin key, InMemory)."

**Comandos de verificación:**
- `cargo check -p vantadb --features server`
- `cargo nextest run --profile audit -p vantadb --features server -E 'test(/encryption_key_ignored/)'`
- `cargo check -p vantadb --features server,encryption`
- `cargo check -p vantadb-server` · `cargo check -p vantadb-server --features encryption`
- `cargo fmt --check` · `cargo clippy -p vantadb --features server --all-targets -- -D warnings`
- `node scripts/docs/check-links.mjs` · `check-docs.mjs` · `gen-index.mjs --check`

## Spec (SDD — decisiones por evidencia)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Re-scope (a)/(b) | (a) superficie server sin wiring / (b) contrato reducido declarando dependencia FIND-249 | ✅ **(b)** — evidencia: FIND-249 diferido por decisión owner (`Backlog.md:384`); contrato pleno imposible sin duplicar FIND-249 (stop condition). La (b) ES "superficie de config/key mgmt + docs honestas + test" del contrato del plan. |
| 2 | Punto de emisión del aviso key-sin-feature | A) `vantadb-server/src/main.rs` (cubre HTTP+MCP del binario, deja fuera `vanta-cli server --http`) / B) **`run_with_hooks` (`bootstrap.rs:310`)** — choke point único del HTTP server: cubre CLI (`cli_handlers/server.rs:310`) + binario (`main.rs:92`); corre tras `validate_auth_config` (precedente de chequeo pre-open) y ANTES del open (visible aunque el open falle) / C) `config.rs` (rechazado por ENC-01 decisión #1: dispara en toda construcción de config) | ✅ **B** — evidencia: callers de `run_with_hooks`; `HTTP_API.md:795-799` documenta `vanta-cli server --http` como ruta primaria de arranque. |
| 3 | Feature forwarding `encryption` en `vantadb-server` | A) no agregarla (el server nunca puede optar; la verificación "ENC-01 cubre la ruta server" queda inalcanzable) / B) **`encryption = ["vantadb/encryption"]`** — aditiva, opt-in, sin default (patrón `tls = ["vantadb/tls"]`) | ✅ **B** — evidencia: RED mecánico `cargo check -p vantadb-server --features encryption` → "does not contain this feature" (2026-10-06); con la feature, `bootstrap.rs:314` → `init.rs` emite el aviso ENC-01. |
| 4 | Alcance del aviso (backend) | A) avisar con cualquier backend / B) **excluir `InMemory`** (sin artefactos on-disk → falso positivo) | ✅ **B** — evidencia: consistencia con ENC-01 decisión #4 (`init.rs:622-624`). |
| 5 | Ruta MCP | A) cubrir también `vantadb-server --mcp` / B) **fuera de alcance** (superficie distinta: `run_stdio_server_auto`, crate `vantadb-mcp`; SRV-10 = "server (HTTP)") | ✅ **B** — evidencia: título del plan Task 70 "(HTTP)"; `main.rs:54-67`; nota ENC-01 en FIND-249 difiere el counterpart core-wide. |
| 6 | Wording | A) mínimo ("not encrypted") / B) **explícito + accionable**: "key is IGNORED … remains PLAINTEXT … use OS/volume-level encryption … FIND-249" | ✅ **B** — pre-mortem #3 (falsa seguridad) + consistencia con ENC-01 decisión #3. |
| 7 | Alcance de docs | A) solo `HTTP_API.md` / B) **HTTP_API + CONFIGURATION (:74, :499) + hardening (:280)** + nota en FIND-249 | ✅ **B** — evidencia: `hardening.md:280` es un "Next Step" que queda stale al cerrar SRV-10; CONFIGURATION es el config reference; ningún edit promete protección. |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. `src/crypto.rs`, `src/storage/vfile.rs` y write paths de storage (WAL/HNSW/VantaFile/KV/text_index/snapshots) NO se tocan — frontera FIND-249.
  2. El aviso NO cambia el flujo del server: solo log a stderr en el arranque (antes del open); sin key o con feature → byte-idéntico.
  3. Sin símbolos públicos Rust nuevos; `tests/api/public-api.txt` (HARD-01) intacto.
  4. La key NO se loggea (solo su presencia) — regla de secretos.
  5. `opencode.jsonc`, master plan y `docs/pipeline-state.json` NO se tocan (WIP ajeno / bookkeeping del orquestador).
- **Comandos de verificación:** los del §Contrato.
- **Deuda pendiente:** FIND-249 (wiring completo, 3-5d, diferido por owner). Ruta MCP sin counterpart (declarada fuera de alcance; la nota ENC-01 la cubre a nivel core al cablear).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** sin deuda nueva (warning de log + feature opt-in + docs + tests). FIND-249 documenta deuda pre-existente, no la introduce.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Aviso positivo + negativos (sin key / InMemory) + feature forwarding verificada (check con/sin feature) + docs sincronizadas sin claim + fmt/clippy/nextest scoped verdes |
| **Commit** | Commit atómico conventional `feat(server):` + `git diff` limpio + verificación mecánica (nunca auto-reporte) |
| **Release** | n/a directo — el commit alimenta el changelog de release-plz; push diferido al final del plan (decisión owner) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius `run_with_hooks`/`validate_auth_config` — usado) + `codebase-memory-mcp_check_index_coverage` (9 paths, sin gaps registrados)
- `cargo check`/`nextest` scoped (loop TDD) + `campaign_verify_cmd` + `pwsh dev-tools/ocr-review.ps1` (OCR delegation al cierre)
- Heavy test lock (`dev-tools/heavy-test-lock.ps1`) para los verify de cierre (nextest workspace)

**Skills cargadas (SDP v3):** `security-and-hardening` (honestidad del control at-rest — trust boundary) · `documentation-and-adrs` (pinned: docs/api + operations) · `api-and-interface-design` (pinned: superficie de config opt-in) · `deprecation-and-migration` (pinned: retiro del claim "SRV-10 on the roadmap") · `source-driven-development` (feature forwarding cargo) · `rust-write-tests` (calidad de tests feature-condicionados) · `test-driven-development` (RED→GREEN) · `incremental-implementation` (slices + verify scoped) · `documentation-skill` (MUST por AGENTS.md para edits en `docs/**`). Base auto: campaign-executor, progreso, ponytail.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — es el objeto de la tarea: eliminar la falsa sensación de seguridad en la superficie server (key silenciosamente ignorada). Checklist `security-and-hardening`: (1) el control inexistente se declara explícitamente al arrancar (no silencio); (2) docs sin claims falsos; (3) sin input de usuario nuevo, sin auth, sin dependencias nuevas; (4) la key NO se loggea (solo presencia). Trust boundary: archivos on-disk (at-rest) + operador que configura la key.
- [x] **PERFORMANCE** — no aplica: comparación de `Option` + 1 log por arranque; no hot path. Regla 9 no dispara.

## Steps

### Step 1 — RED→GREEN: aviso key-sin-feature en `run_with_hooks` + tests

- **Archivos:** `src/server/bootstrap.rs`
- **Acción:** RED (feature-condicionados `#[cfg(not(feature = "encryption"))]`): tests que fijan el aviso → no compila (`encryption_key_ignored_notice` inexistente). GREEN: `encryption_key_ignored_notice(&Config) -> Option<&'static str>` + `warn_if_encryption_key_ignored(&Config)` (privados, cfg'd) + call en `run_with_hooks` tras `validate_auth_config`.
- **Verify:** `cargo nextest run --profile audit -p vantadb --features server --lib -E 'test(/encryption_key_ignored/)'` → RED luego GREEN 3/3
- **Evidencia:** ✅ RED (main tree, árbol aún compilable): `3 tests run: 2 passed, 1 failed` — positivo falla con "key set without the `encryption` feature must warn" (stub `None`), por la razón correcta; negativos pasan. GREEN (worktree de verificación — ver Notas de entorno): `3 tests run: 3 passed`.
- **Estado:** ✅ COMPLETED

### Step 2 — Feature forwarding `encryption` en `vantadb-server` + checks con/sin feature

- **Archivos:** `vantadb-server/Cargo.toml`
- **Acción:** agregar `encryption = ["vantadb/encryption"]` (patrón `tls`); verificar `cargo check -p vantadb-server` y `cargo check -p vantadb-server --features encryption` (hoy RED: "does not contain this feature").
- **Verify:** ambos check exit 0
- **Evidencia:** ✅ RED pre-fix (main tree): `cargo check -p vantadb-server --features encryption` → `error: the package 'vantadb-server' does not contain this feature: encryption`. Post-fix (worktree): `cargo check -p vantadb-server` exit 0 (28.6s) · `cargo check -p vantadb-server --features encryption` exit 0 (26.6s) · `cargo check -p vantadb --features server,encryption` exit 0 (18.0s).
- **Estado:** ✅ COMPLETED

### Step 3 — Docs honestas

- **Archivos:** `docs/api/HTTP_API.md` (:772-774), `docs/user/operations/CONFIGURATION.md` (:74, :499), `docs/user/operations/hardening.md` (:280) — nota en `Backlog.md` OMITIDA (archivo bajo edición concurrente de WIRE-13; ver Notas)
- **Acción:** actualizar sin prometer protección: el aviso al arrancar cuando key set; wiring = FIND-249; guía OS/volume.
- **Verify:** `node scripts/docs/check-links.mjs` + `check-docs.mjs` + `gen-index.mjs --check` exit 0
- **Evidencia:** ✅ `check-links` exit 0 · `check-docs` exit 0 · `gen-index --write` regeneró `docs/index.md`+`llms.txt` → `gen-index --check` exit 0 · `validate-docs-coverage.ps1` → "0 gaps" (main tree).
- **Estado:** ✅ COMPLETED

### Step 4 — Verify full scoped + OCR + review P2-01 (adversarial) + commit local

- **Archivos:** `docs/dev/tasks/SRV-10.md`
- **Acción:** verify full (fmt + clippy workspace + nextest workspace con heavy lock + validate-docs-coverage) + OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`); review P2-01 tier **Adversarial** (`docs/api/**` matchea → fork a `vanta-review`); registrar veredicto en §Review; commit **LOCAL** `feat(server):` solo con archivos propios (nunca push).
- **Verify:** `git show --stat HEAD` limitado a archivos propios; veredicto en §Review
- **Evidencia:** ⬜ pendiente
- **Estado:** ⬜ PENDING

## Dependencias

- F5 — última tarea de la fase. DUR-02 ✅ (evidencia del gap). ENC-01 ✅ (aviso core, base). FIND-249 diferido (frontera declarada).
- WIRE-13 en vuelo (parser — área disjunta; pathspec estricto en el commit).
- nextTask: F6 / cierre del plan (lo decide el orquestador).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** sesión fresca P2-01 (sin participación en SRV-10) — tier **Adversarial** (`docs/api/**` → HARD-02). SKILLS_CARGADAS: `doubt-driven-development`, `code-review-and-quality`; SDP: `ponytail` (lente), `rust-write-tests` (tests feature-condicionados), `security-and-hardening` (checklist OCR de la superficie). Revisión sobre worktree `%TEMP%\opencode\vantadb-srv10-verify` (HEAD a25c9314 + SRV-10) y árbol principal; SHA256 de los 5 archivos SRV-10 = main tree (worktree fiel).
- **Enfoque:** re-scope (b) contra evidencia mecánica; frontera anti-FIND-249; punto de emisión y cfg del aviso; forwarding de feature (patrón `tls`); honestidad de docs (sin claim de protección); falsos positivos/negativos (InMemory, sin key, feature ON); HARD-01; re-ejecución puntual de la evidencia citada.
- **Cómo se probó:** (re-ejecutado por el revisor; nunca auto-reporte)
  - `cargo nextest run --profile audit -p vantadb --features server --lib -E 'test(/encryption_key_ignored/)'` → **3 tests run: 3 passed**.
  - `cargo check -p vantadb --features server` ✅ · `-p vantadb --features server,encryption` ✅ · `-p vantadb-server` ✅ · `-p vantadb-server --features encryption` ✅ (todos exit 0).
  - `cargo clippy -p vantadb --features server --all-targets -- -D warnings` ✅ · `cargo fmt --check -p vantadb` ✅.
  - Premisa "default sin encryption" verificada: `cargo tree -p vantadb-server -e features` → 0 aristas `encryption`; `src/lib.rs:55-57` gatea `pub mod crypto`; únicos consumidores de `config.encryption_key` = `src/crypto.rs:198` + `src/storage/engine/init.rs:621`, ambos feature-gated → la key es realmente inerte sin la feature (el diagnóstico del gap es honesto).
  - `git diff HEAD --stat -- tests/api/public-api.txt` → vacío (HARD-01 intacto); fns nuevas privadas (`src/server/bootstrap.rs:135,156`).
  - Docs: `check-links` ✅ · `check-docs` ✅ · `gen-index --check` ❌ en main tree — **causa ajena, atribución completa** (H1).
  - OCR advisory verificado: el preview asigna grupos de reglas dedicados a `src/server/bootstrap.rs` (group 4) y `vantadb-server/Cargo.toml` (group 5); checklist aplicado: sin Critical/High/Medium (sin `unwrap/expect` en prod, sin key en logs, feature aditiva sin wildcards, defaults intactos).
- **Hallazgos:**
  - **H1 — Required (acción mecánica pre-commit; causa ajena a SRV-10).** `node scripts/docs/gen-index.mjs --check` FALLA hoy en el árbol principal. Atribución probada: la única discrepancia esperado-vs-committed es la fila IQL (`docs/index.md:61`, `docs/api/index.md`, `llms.txt`), por el cambio de `description` en `docs/api/IQL.md` (mtime 06:40:30 — WIRE-13 en vuelo) posterior a la regeneración de SRV-10 (06:37:28). Prueba: regeneración en copia scratch con `.gitignore` + git (excluye el ignorado `docs/user/discord/todo.md`) → diff contra el árbol = SOLO la fila IQL en los 3 archivos; conteos 1572/1108 y filas SRV-10/WIRE-13 idénticos. Acción: re-ejecutar `gen-index --write` + `--check` en la ventana de commit (queda verde al incluir IQL v4) o al cierre del plan; no editar a mano. No bloquea el contenido de SRV-10.
  - **H2 — Optional.** `docs/user/operations/CONFIGURATION.md:74,499`: "the server warns at startup" sin acotar a HTTP; `vantadb-server --mcp` (fuera de alcance declarado, Spec #5) ignora la key y no avisa. Recomendado: acotar a "the HTTP server" o añadir nota MCP.
  - **H3 — Optional.** `docs/api/HTTP_API.md:772-777`: "The server warns at startup when `VANTADB_ENCRYPTION_KEY` is set" — con la feature compilada el aviso es del engine ENC-01 en el open y con InMemory ninguno de los dos avisa. No promete protección (correcto); considerar "during startup" para no sobre-especificar.
  - **H4 — Optional (limitación aceptada).** Los tests fijan el helper (`src/server/bootstrap.rs:799-849`), no el call-site (`:353`): borrar la llamada no rompe tests; mismo trade-off que ENC-01 (helper-only). Aceptable para un warning; un test de arranque lo cubriría a costa de abrir storage/puerto.
  - **H5 — Nit.** `docs/dev/tasks/SRV-10.md` §Blast Radius (:48) menciona "nota en Backlog.md (FIND-249)" pero Step 3 la omitió (justificado: WIP WIRE-13) — corregir la fila para consistencia del task file.
  - **H6 — Nit.** `CONFIGURATION.md:499` "enable it via `vantadb-server --features encryption`" es shorthand de build (el §7 lo aclara); considerar `cargo build -p vantadb-server --features encryption`.
- **Alternativas evaluadas (brainstorm).** Punto de emisión: (i) counterpart core-wide en `init.rs` (simétrico con ENC-01, cubriría MCP/embedders; mayor blast radius y fuera del scope "(HTTP)") — diferido: la propia nota ENC-01 de `Backlog.md:384` ya lo agenda "al cablear"; (ii) `vantadb-server/src/main.rs` (dejaría fuera `vanta-cli server --http`); (iii) `config.rs` (rechazado: dispara en toda construcción de config). El choke point `run_with_hooks` elegido cubre las dos rutas HTTP reales (`cli_handlers/server.rs:310`, `main.rs:92`), corre pre-open y tras `validate_auth_config` — mínimo honesto. Cfg complementario correcto: el aviso SRV-10 es exactamente el lado `not` del cfg de ENC-01 en el mismo crate → sin doble aviso ni gap por unificación de features. Re-scope: plan Task 70 (:2007) lo manda ("re-scope obligatorio en DISCOVERY"); nada de write paths storage/WAL/HNSW/crypto.rs fue tocado (frontera FIND-249 respetada).
- **Veredicto:** ✅ **APPROVE** — 0 Critical / 0 High / 0 Medium. H1 es una acción mecánica de la ventana de commit con causa totalmente ajena (WIRE-13), evidencia incluida; H2–H6 opcionales/nits. Contrato: 5/6 grupos de comandos re-ejecutados verdes por el revisor; el 6.º (`gen-index --check`) falla solo por la fila IQL de WIRE-13.

## Notas

- **Contexto (FIND-249/DUR-02):** 0/6 artefactos on-disk cifrados con la feature activa; primitivas AES-256-GCM OK (consumidor real: envelope del proxy). Decisión owner 2026-10-04: avisar ahora (ENC-01 ✅), cablear luego (FIND-249).
- **Riesgo aceptado:** el aviso se emite 1 vez por arranque con key configurada (sin estado global — mismo criterio que ENC-01).
- **Gate D:** no disparado — re-scope resuelto por evidencia (decisión owner FIND-249); superficie aditiva opt-in; sin símbolos públicos nuevos; blast radius ≤10 archivos; sin hot path.
- **Gate V:** no disparado — sin fallas de verify repetidas.
- WIP ajeno en el árbol (`opencode.jsonc`, master plan, `Backlog.md`/`src/parser/mod.rs`/`src/query.rs`/`src/executor.rs` de WIRE-13) no se stagea.
- **Entorno (lock del bin, FIND-177/MEMG-08):** el MCP server de memoria vivo (`vanta-mcp-local.ps1` → `target/debug/vanta-cli.exe server --mcp`) lockea `vanta-cli.exe`/`vantadb-server.exe` → todo relink falla ("Acceso denegado os error 5"). NO se mató el proceso (precedente MEMG-03). Verificación ejecutada en worktree aislado `%TEMP%\opencode\vantadb-srv10-verify` (HEAD + los 6 archivos de esta tarea, `CARGO_TARGET_DIR` compartido con el main tree): GREEN 3/3 · suite lib+server **2509/2509** · clippy scoped + workspace all-features ✅ · fmt ✅ · OCR preview. El worktree se elimina al cierre.
- **WIRE-13 mid-flight:** el árbol principal no compila (Step 1 RED deliberado: `src/parser/mod.rs`+`src/query.rs`) y `src/executor.rs` está sin `cargo fmt` → el pre-commit hook (fmt+clippy whole-tree) puede fallar por archivos ajenos; el commit se intenta en ventana verde; si no, `SKIP_CLIPPY=1` documentado (la evidencia de worktree cubre clippy/fmt sobre contenido idéntico).
- **OCR delegation (advisory):** preview `ocr-review.ps1 -Format json` → Rule Group 1 (`src/server/bootstrap.rs`) + Group 2 (`vantadb-server/Cargo.toml`) revisados: 0 Critical/High/Medium (sin `unwrap`/`expect`/`panic` nuevos en producción — los `expect` viven en tests; sin secretos loggeados — el aviso NO incluye la key; feature aditiva sin wildcards; sin locks/async nuevos).

## RESULTADO §7 (contrato de retorno — pipeline-full)

```
RESULTADO: ⬜ pendiente
```
