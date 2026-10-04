---
title: "TASK ENC-01: Cifrado honesto: aviso al activar + docs (FIND-249 parte 1)"
kind: task
description: "Warning accionable al activar la feature `encryption` + `VANTADB_ENCRYPTION_KEY` (0/6 artefactos on-disk cifrados, DUR-02) + docs sin ambigüedad + test que fija el aviso. Cableado completo: FIND-249"
---

# TASK ENC-01: Cifrado honesto: aviso al activar + docs (FIND-249 parte 1)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 72, Wave F0 — addendum)
- **Fuente:** decisión owner 2026-10-04 (question) + FIND-249 (DUR-02) + plan Task 72
- **Esfuerzo:** 🟢 2-4h | **Appetite:** 1d
- **Prioridad:** 🟠
- **Tipo:** Mixto (Rust core + Docs)
- **Turns estimados:** 8-12
- **Creado:** 2026-10-04T14:07Z | **last-synced:** 2026-10-04T14:45Z
- **Estado:** ⏳ IN PROGRESS (cierre en curso)
- **Incógnitas (uphill):** 0 — punto de activación identificado (`StorageEngine::open_with_config`), mecanismo anti-ruido decidido (gating por entorno), wording fijado con pre-mortem #3
- **Pendientes (downhill):** 0 steps (4/4 ✅; resta commit de cierre + registro)
- **Campaign ID:** master-plan-0.9.0-20261004

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `StorageEngine::open_with_config` (`src/storage/engine/init.rs:29`) — 138 callers: `Embedded::open_with_config` (`src/sdk/builder.rs:113`), CLI `open_database`/`open_embedded` (`src/cli_handlers/db.rs:14,24`), python bindings (`vantadb-python/src/lib.rs:163`), server/MCP, benches y tests. |
| Callees | init.rs → `crate::config::Config` (`encryption_key: Option<String>`, `backend_kind`), `tracing` (`warn!`). Sin dependencias nuevas. |
| Implicaciones | **Contrato público:** sin símbolos nuevos (helpers privados del módulo `init`); el aviso es comportamiento de log, no API. **Comportamiento:** al abrir con feature `encryption` + key activos (backend on-disk) se emite 1 warning por open; sin feature/key, byte-idéntico. **Performance:** nula (comparación de `Option` en el open, no hot path). **Migración de datos:** ninguna. **Tests:** nuevos feature-gated en `tests/init.rs`; suite sin feature intacta. **Docs:** `CONFIGURATION.md` (:73 + §7) y `FEATURES.md` (:47) + rustdoc de `config.rs`/`lib.rs`. |

**PROHIBIDO tocar:** `opencode.jsonc` (WIP ajeno), `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (bookkeeping del orquestador), `docs/pipeline-state.json`, `docs/api/**` / `docs/user/glossary/**` (áreas de DOCS-F1/F2 recién cerradas), `release-plz.toml` / `PUBLISH.md`. WIP ajeno del árbol no se stagea.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `src/storage/engine/init.rs` (608L) — `open_with_config` (`:29-146`), `init_storage` (`:148-293`), `init_indexes` (`:295-376`), `recover_state` (`:378-607`). Imports (`:3-20`): `BackendKind` ya importado; `tracing::info` importado. Punto de inserción: call en `open_with_config` tras `config.unwrap_or_default()`; helpers a nivel módulo tras el `impl` (fin del archivo).
  - `src/storage/engine/tests/init.rs` (764L) — convención: `use super::super::*;`, tests feature-gated con `#[cfg(feature = ...)]`, `tempfile::tempdir()` disponible, helper local `open_disk_engine_with_wal` (`:499`). Los tests nuevos se anexan al final.
- **Archivos leídos (secciones funcionales completas):**
  - `src/crypto.rs` (verbatim CodeGraph) — `Cipher::from_env` (`:196-202`, lee `Config::default().encryption_key`), `EncryptionStream` (`:392-431`), tests (`:719-783`). Consumidor real: `vanta-proxy/src/envelope.rs:125` (fuera del crate).
  - `src/config.rs` — campo `encryption_key` (`:225`, `:856-861`), `Default` (`:323`), `StorageCfg` (`:417`), carga env (`:1380-1386`), builder `with_encryption` (`:1657-1664`).
  - `src/lib.rs:45-74` — doc del módulo `crypto` (`:55-57`, `#[cfg(feature = "encryption")]`).
  - `src/sdk/builder.rs:88-128` — `Embedded::open_with_config` delega en `StorageEngine::open_with_config` (`:113-116`).
  - `src/console.rs:100-129` — `init_logging` instala subscriber (stderr, `RUST_LOG` default `info`) → el `warn!` es visible en el CLI.
  - `docs/user/operations/CONFIGURATION.md` — fila `encryption_key` (`:73`, ya honesta vía DUR-02), §7 Cargo Features (`:475-495`, **sin fila `encryption`**).
  - `docs/dev/architecture/FEATURES.md:35-74` — fila `encryption` (`:47`, ya honesta vía DUR-02).
- **Archivos referenciados hacia dentro (imports/dependencias):** `init.rs` → `crate::config::Config`, `crate::storage::engine::{BackendKind,...}`, `tracing`. Sin imports nuevos (los helpers usan lo ya importado).
- **Referencias entrantes (grep/CodeGraph):** `StorageEngine::open_with_config` 138 callers (lista en Blast Radius); `encryption_key` consumidores: `src/config.rs` (definición), `src/crypto.rs:198` (solo `from_env`), ningún write path de storage. `warn_encryption_not_wired` es símbolo nuevo privado (0 referencias entrantes por diseño).
- **Veredicto impacto:** **BAJO** — warning de log en el open; sin símbolos públicos, sin cambios de formato on-disk, sin hot path, sin migración. Riesgos del pre-mortem mitigados: (1) builds con feature → `cargo check/clippy/nextest --features encryption` en verify; (2) ruido → gating por entorno (solo feature+key) + test negativo; (3) wording → texto explícito "NOT yet wired"/"PLAINTEXT" fijado por test.

## Contrato

"Al activar `encryption` (feature + key, backend on-disk), `StorageEngine::open_with_config` emite un warning accionable (texto explícito 'NOT yet wired' + 'PLAINTEXT' + referencia FIND-249) + `CONFIGURATION.md` y `FEATURES.md` lo dicen sin ambigüedad (referencia FIND-249) + tests feature-gated que fijan el aviso (positivo + negativos sin-key/InMemory); `cargo fmt --check` + `cargo clippy -p vantadb --features encryption --all-targets -- -D warnings` + `cargo nextest run --profile audit -p vantadb --features encryption -E 'test(/encryption_notice/)'` y sin feature verdes + gates docs (check-links/check-docs/gen-index) 0."

## Spec (SDD — decisiones por evidencia)

> La tarea NO agrega símbolos públicos nuevos (helpers privados; el aviso es log). Decisiones resueltas por evidencia:

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Punto de emisión | A) `Config::default()` (`config.rs:1380`, donde se lee la env) — dispara en toda construcción de config (incl. `Cipher::from_env()` del proxy, que sí cifra sus envelopes: falso positivo) / B) **`StorageEngine::open_with_config` (`init.rs:29`)** — choke point real de activación: 138 callers, cubre CLI (`open_database`), SDK (`Embedded`), python, server / C) `Embedded::open_with_config` — dejaría fuera el path CLI `open_database` (`cli_handlers/db.rs:14`) | ✅ **B** — evidencia: `cli_handlers/db.rs:14` vs `:24` (ambos paths CLI), `builder.rs:113` (Embedded delega), `python/lib.rs:163` |
| 2 | Mecanismo anti-ruido (pre-mortem #2) | A) `Once` por proceso (estado global no-reseteable; complica tests determinísticos) / B) **gating por entorno: solo se emite si feature + key están activos** — 1 aviso por open = 1 por activación; ningún test de la suite setea la key; CI no setea `VANTADB_ENCRYPTION_KEY` / C) ambos | ✅ **B** — "emitir una vez / gating por entorno" (pre-mortem): se elige gating; N opens con key = N avisos, aceptado y documentado |
| 3 | Wording | A) mínimo ("not encrypted") / B) **explícito + accionable**: "encryption is NOT yet wired … remain PLAINTEXT … Do not rely on this feature … use OS/volume-level encryption … See FIND-249" | ✅ **B** — pre-mortem #3 + Gate Justificación ("falsa sensación de seguridad") |
| 4 | Backend `InMemory` | A) avisar igual / B) **excluir** — no hay artefactos on-disk; el aviso sería un falso positivo en `:memory:` | ✅ **B** — evidencia: `init.rs:38-44` (branch InMemory sin WAL/vfiles en disco) |
| 5 | Alcance de docs | A) solo `CONFIGURATION.md`/`FEATURES.md` (ya honestos por DUR-02) / B) **+ rustdoc que aún miente**: `config.rs:856-860` ("storage files are transparently encrypted"), `config.rs:1657-1660` (builder), `lib.rs:55` ("at-rest encryption for storage files") + fila `encryption` en §7 | ✅ **B** — mismos claims falsos, mismo propósito (honestidad); 4 ediciones de 1-3 líneas |
| 6 | ¿Cablear cifrado ahora? | A) wiring (3-5d, toca mmap/WAL/formatos) / B) **NO — FIND-249** (decisión owner: "avisar ahora, cablear luego") | ✅ **B** — fuera de scope por contrato |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. `src/crypto.rs` y `src/storage/vfile.rs` NO se tocan (primitivas intactas; fix DUR-02 preservado).
  2. El aviso NO cambia la semántica del open: el engine abre normalmente (solo log).
  3. Sin símbolos públicos nuevos; `tests/api/public-api.txt` (HARD-01) intacto.
  4. Sin feature `encryption`: comportamiento byte-idéntico (helpers compilados solo con la feature).
  5. `wal.rs` / `sdk/api/memory.rs` NO se tocan (WIP de otros workers).
- **Comandos de verificación:** `cargo check -p vantadb --features encryption` · `cargo nextest run --profile audit -p vantadb --features encryption -E 'test(/encryption_notice/)'` · `cargo nextest run --profile audit -p vantadb -E 'test(/init/)'` (sin feature) · `cargo fmt --check` · `cargo clippy -p vantadb --features encryption --all-targets -- -D warnings` · `node scripts/docs/check-links.mjs` + `check-docs.mjs` + `gen-index.mjs --check`.
- **Deuda pendiente:** FIND-249 (wiring completo, 3-5d, diferido por owner). Nota: "key seteada sin feature compilada" no se avisa (fuera del contrato — feature+key; candidato a nota en FIND-249).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** sin deuda nueva (warning de log + docs + tests; FIND-249 documenta deuda pre-existente, no la introduce).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Aviso emitido en el punto de activación (positivo + negativos) + docs/rustdoc sin ambigüedad + tests feature-gated + fmt/clippy/nextest scoped con y sin feature verdes |
| **Commit** | Commit atómico conventional `fix(security):` + `git diff` limpio + verificación mecánica (nunca auto-reporte) |
| **Release** | n/a directo — el commit alimenta el changelog de release-plz (nota de transparencia); push diferido al final del plan (decisión owner) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius init/builder — usado) + `codebase-memory-mcp_check_index_coverage` (5 paths, sin gaps registrados) + grep (consumidores `encryption_key`)
- `cargo check -p vantadb --features encryption` + `cargo nextest run --profile audit -p vantadb --features encryption -E 'test(/encryption_notice/)'` (loop TDD)
- `campaign_verify_cmd` (verify mecánico) + `pwsh dev-tools/ocr-review.ps1` (OCR delegation al cierre)

**Skills cargadas (SDP v3):** `security-and-hardening` (honestidad de control de seguridad — trust boundary at-rest) · `documentation-skill` (edits en `docs/**` + frontmatter/gates) · `rust-write-tests` (calidad de tests feature-gated) · `test-driven-development` (RED→GREEN) · `incremental-implementation` (slices + verify scoped) · `source-driven-development` (verificación del API de tracing/tracing-subscriber). Base auto: campaign-executor, progreso, ponytail. SDP: 6 cargadas + base; `writing-*`/`context-engineering` excluidas por no aplicar (task 🟢 acotada).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — es el objeto de la tarea: eliminar la falsa sensación de seguridad de `encryption`. Checklist `security-and-hardening`: (1) el control inexistente se declara explícitamente al activarse (no silencio); (2) docs/rustdoc sin claims falsos; (3) sin input de usuario nuevo, sin auth, sin dependencias nuevas, sin secretos loggeados (la key NO se loggea — solo su presencia). Trust boundary: archivos on-disk (at-rest).
- [x] **PERFORMANCE** — no aplica: el open no es hot path (una comparación de `Option` + posible `warn!` una vez por open). Regla 9 no dispara.

## Steps

### Step 1 — RED→GREEN: aviso en el punto de activación + tests

- **Archivos:** `src/storage/engine/init.rs`, `src/storage/engine/tests/init.rs`
- **Acción:** test RED (feature-gated, `#[cfg(feature = "encryption")]`) que abre un engine con key + backend on-disk capturando el subscriber (`tracing::subscriber::with_default` + writer compartido) → falla (no hay aviso); luego: `encryption_not_wired_notice(&Config) -> Option<&'static str>` + `warn_encryption_not_wired(&Config)` (privados, `#[cfg(feature = "encryption")]`) + llamada en `open_with_config` (`#[cfg(feature = "encryption")] warn_encryption_not_wired(&config);`).
- **Verify:** `cargo nextest run --profile audit -p vantadb --features encryption -E 'test(/encryption_notice/)'` → RED luego GREEN
- **Evidencia:** ✅ RED verificado en HEAD pre-fix: `test_encryption_notice_emitted_when_active` FAILED — "warning must say encryption is not wired; got: <empty>" (2 passed / 1 failed). Fix: `encryption_not_wired_notice` + `warn_encryption_not_wired` (privados, feature-gated) + call en `open_with_config` (`init.rs`) → GREEN 3/3 passed.
- **Estado:** ✅ COMPLETED

### Step 2 — Tests negativos + scoped con/sin feature

- **Archivos:** `src/storage/engine/tests/init.rs`
- **Acción:** tests que fijan la ausencia del aviso: (a) key ausente + backend on-disk → sin aviso; (b) key + `BackendKind::InMemory` → sin aviso (no hay artefactos at-rest). Correr scoped con y sin feature + `cargo check --features encryption`.
- **Verify:** `cargo nextest run --profile audit -p vantadb --features encryption -E 'test(/encryption_notice/)'` ✅ + `cargo nextest run --profile audit -p vantadb -E 'test(/init/)'` ✅ + `cargo check -p vantadb --features encryption` ✅
- **Evidencia:** ✅ `cargo check -p vantadb --features encryption` exit 0 (21s) · scoped sin feature `test(/init/)` → 51/51 · suite completa con feature → **2575/2575 passed** (251.9s) · suite completa sin feature → **2549/2549 passed** (241.7s).
- **Estado:** ✅ COMPLETED

### Step 3 — Docs honestas + rustdoc

- **Archivos:** `docs/user/operations/CONFIGURATION.md` (fila `:73` + fila nueva §7), `docs/dev/architecture/FEATURES.md` (fila `:47`), `src/config.rs` (rustdoc campo + builder), `src/lib.rs` (doc módulo `crypto`)
- **Acción:** declarar el aviso de activación en las filas ya honestas (DUR-02) + agregar fila `encryption` en §7 (hoy ausente) + corregir los 3 rustdoc que aún afirman cifrado at-rest.
- **Verify:** `node scripts/docs/check-links.mjs` + `check-docs.mjs` + `gen-index.mjs --check` exit 0
- **Evidencia:** ✅ `CONFIGURATION.md:73` (+ aviso de activación) + §7 (fila `encryption` nueva) + `FEATURES.md:47` (+ aviso) + rustdoc `config.rs` (campo `:856`, builder `:1657`) + `lib.rs:55` corregidos; `check-links` exit 0 · `check-docs` exit 0 (GATING all clear) · `gen-index --write` regeneró `docs/index.md` + `llms.txt` → `gen-index --check` exit 0 · `validate-docs-coverage.ps1` 0 gaps.
- **Estado:** ✅ COMPLETED

### Step 4 — Verify full scoped + OCR + review P2-01 + commit local

- **Archivos:** `docs/dev/tasks/ENC-01.md`
- **Acción:** `cargo fmt --check` + `cargo clippy -p vantadb --features encryption --all-targets -- -D warnings` + nextest scoped (con y sin feature) + gates docs; OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`); review P2-01 (tier **Adversarial** — `src/storage/**` matchea HARD-02 → fork a `vanta-review`); registrar veredicto en §Review; commit **LOCAL** `fix(security): ENC-01 — ...` solo con archivos propios (nunca push).
- **Verify:** `git show --stat HEAD` limitado a archivos propios; veredicto en §Review
- **Evidencia:** ✅ `cargo fmt --check` exit 0 · `cargo clippy -p vantadb --features encryption --all-targets -- -D warnings` exit 0 · nextest scoped en 4 escenarios de entorno (limpio / `VANTADB_ENCRYPTION_KEY` / `VANTADB_BACKEND=memory` / ambas) → 3/3 cada uno · gates docs 0 · OCR delegation (Rule Groups 1-2): 0 Critical/High · review P2-01 `vanta-review` ✅ **APPROVE** (R1 changes-required por hermeticidad → fix → R2 approve re-verificado). Commit local `fix(security):` (ver RESULTADO) + commit de cierre `docs(avance):`. Nit §7 incorporado post-R2 (texto exacto recomendado por el reviewer, registrado en §Review).
- **Estado:** ✅ COMPLETED

## Dependencias

- F0 — última tarea de la fase (21/21 previas ✅). DUR-02 ✅ (evidencia del gap).
- nextTask: checkpoint F0 / F1 (lo decide el orquestador).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review — P2-01, contexto fresco (no participó de la implementación); tier Adversarial (HARD-02). Ciclo: R1 🔴 (1 Required + 2 Low) → fix → R2 ✅. Sesión reviewer: `ses_ef8aac205ffek6RD7sIQZnc1mA`.
- **Enfoque:** Punto de emisión correcto y verificado: `StorageEngine::open_with_config` (init.rs:32-35) cubre CLI raw/embedded, SDK, Python, MCP, server (bootstrap.rs:296), desktop/WASM; `StorageEngine::open` delega (init.rs:24-26). Alternativa `Config::default()` rechazada con evidencia (falso positivo en `Cipher::from_env`, crypto.rs:196-202). Wording explícito/accionable; sin API pública; invariantes intactos.
- **Cómo se probó:** scoped `cargo nextest --profile audit -p vantadb --features encryption -E "test(/encryption_notice/)"` en 4 escenarios de entorno — limpio, `VANTADB_ENCRYPTION_KEY`, `VANTADB_BACKEND=memory`, ambas → 3/3 exit 0 en cada uno (hermeticidad cerrada); RED reproducido en R1 vía worktree HEAD adcd5e1c (2 passed/1 failed, "got: <empty>", exit 100); R2: fmt exit 0 · clippy(--all-targets -D warnings) exit 0 · check sin feature exit 0 · check-links/check-docs/gen-index --check exit 0; `public-api.txt` intacto. Suites completas 2575/2549 declaradas en R1, no re-ejecutadas.
- **Hallazgos:** R1 [Required] tests no herméticos al entorno (`VANTADB_ENCRYPTION_KEY` ⇒ fallaba `silent_without_key`; `VANTADB_BACKEND=memory` ⇒ fallaba `emitted_when_active`) — **cerrado**: pinneo explícito de `backend_kind`+`encryption_key` (tests/init.rs:820-840, :875-879, :896) y reproducción de los 4 escenarios verde. R1 [Low] key-sin-feature no avisa — **cerrado**: nota registrada en FIND-249 (`docs/dev/Backlog.md`). R1 [Low/Nit] docs sin excepción InMemory — **cerrado** en CONFIGURATION.md:73 y FEATURES.md:47; el residual §7 (CONFIGURATION.md:495) fue **incorporado post-R2** por el implementador con el texto exacto recomendado por el reviewer (transparencia: cambio posterior al verdicto, dentro de la dirección aprobada).
- **Veredicto:** ✅ **APPROVE** — contrato mecánico verde y hermético en los 4 escenarios; producción solo suma el warning + comentario; listo para commit local (Step 4, sin push).

## Notas

- **Contexto (DUR-02):** 0/6 artefactos on-disk cifrados con la feature activa (WAL/HNSW/VantaFile/backend-KV/text_index/snapshots en plaintext); las primitivas AES-256-GCM funcionan (consumidor real: envelope del proxy). Decisión owner 2026-10-04: **avisar ahora, cablear luego** (FIND-249).
- **Riesgo aceptado:** cada `open` con key configurada re-emite el aviso (gating por entorno, sin `Once`); en la práctica = 1 aviso por activación/proceso.
- **Gate D:** no disparado — sin símbolos públicos nuevos, sin cambio de semántica pública (log), sin ambigüedad de contrato, blast radius ≤10 archivos, sin hot path.
- WIP ajeno en el árbol (`opencode.jsonc`) no se stagea.

## RESULTADO §7 (contrato de retorno — pipeline-full)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: pendiente (commit fix(security) local; se registra en el commit de cierre docs(avance))
ARCHIVOS: src/storage/engine/init.rs, src/storage/engine/tests/init.rs, src/config.rs, src/lib.rs, docs/user/operations/CONFIGURATION.md, docs/dev/architecture/FEATURES.md, docs/dev/Backlog.md, docs/dev/tasks/ENC-01.md, docs/index.md, llms.txt
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:no | P:sin símbolos públicos nuevos · D:contrato explícito · V:verde 4 escenarios · C:sin colaterales
SKILLS_CARGADAS: security-and-hardening, documentation-skill, rust-write-tests, test-driven-development, incremental-implementation, source-driven-development (base auto: campaign-executor, progreso, ponytail)
```
