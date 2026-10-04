---
title: "TASK DUR-02: Auditoría cobertura AES (encryption) — WAL / text_index / HNSW / edge_index / snapshots"
kind: task
description: "Mapa artefacto→cifrado con evidencia de código + probe tmpdir: ningún artefacto on-disk aplica AES-256-GCM (feature sin cablear); fix pequeño aplicado (encryption_stream ignoraba self.cipher) + FIND-249"
---

# TASK DUR-02: Auditoría cobertura AES (`encryption`) — WAL / text_index / HNSW / edge_index / snapshots

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 12, Wave F0)
- **Fuente:** `docs/dev/Backlog.md` (fila DUR-02, verificado HEAD 2026-10-01) + plan Task 12
- **Esfuerzo:** 🟡 1-2d | **Appetite:** 2d
- **Prioridad:** 🟠
- **Tipo:** Rust core (auditoría `src/crypto.rs`/`src/storage/vfile.rs` + docs) — Mixto Rust+Docs
- **Turns estimados:** 15-25
- **Creado:** 2026-10-04T18:30Z | **last-synced:** 2026-10-04T18:30Z
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 — resueltas en Discovery (cobertura real determinada por grep exhaustivo + probe runtime; clasificación de gaps decidida con evidencia)
- **Pendientes (downhill):** 4 steps (Steps 1-2 ✅; 3-6 pendientes)
- **Campaign ID:** master-plan-0.9.0-20261004

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/crypto.rs` (`Cipher`/`EncryptionStream`/`EncryptionConfig`) — único consumidor real hoy: `vanta-proxy/src/envelope.rs:125` (`Cipher::from_env` + `derive_namespace`). `src/storage/vfile.rs` (`with_cipher`/`encryption_stream`): **0 callers** en todo el workspace (grep verificado). |
| Callees | `crypto.rs` → `aes-gcm 0.11.1` + `ring` (PBKDF2/HKDF) + `rand`; `vfile.rs` → `crate::crypto` (feature-gated); write paths auditados: `storage/engine/init.rs`, `wal.rs`/`wal_sharded.rs`, `index/port_impl.rs`+`index/serialize/file.rs`, `backends/fjall_backend.rs`, `text_index.rs`, `edge_index.rs`, `storage/engine/mod.rs` (snapshots) |
| Implicaciones | **Contrato público:** el fix de `encryption_stream` corrige una API pública (feature `encryption`, opt-in) para que use el cipher adjunto (`with_cipher`) en vez de resolver la env var por su cuenta — sin cambios de firma. **Comportamiento:** feature `encryption` sigue sin cifrar nada on-disk (FIND-249, decisión de diseño pendiente). **Docs:** claims ajustados a la realidad (CONFIGURATION.md, FEATURES.md). **Performance:** nula (código dormido sin callers). **Migración de datos:** ninguna. **Tests existentes:** `crypto.rs` tests intactos; el test nuevo es feature-gated. |

**PROHIBIDO tocar:** `opencode.jsonc` (WIP ajeno), `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (bookkeeping del orquestador), `docs/pipeline-state.json`, WIP de otros workers (`DUR-01` edita `src/wal.rs` — **solo lectura**; `DUR-03` edita el put path de `src/sdk/api/memory.rs` — **no tocar**).

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `src/crypto.rs` (783L) — `Cipher` (AES-256-GCM, PBKDF2 para passphrase, HKDF `derive_namespace`), `EncryptionStream` (framing `[4B len][12B nonce][ct+tag]`), `EncryptionConfig`/`resolve_cipher`, `Cipher::from_env` (lee `Config::default().encryption_key`), tests (roundtrips, KDF, oversized frame).
  - `src/storage/vfile.rs` (914L) — campo `cipher: Option<Cipher>` (`:126`, feature-gated), `with_cipher` (`:439`), `cipher()` (`:446`), `encryption_stream` (`:459-463`).
  - `docs/user/operations/CONFIGURATION.md` (514L) — fila `encryption_key` (`:73`), §7 Cargo Features.
  - `src/edge_index.rs` (63L) — `DashSet<(u128,u128)>` in-memory, sin persistencia.
  - `docs/api/HTTP_API.md:690-703` (sección "honestly behind") — ya declara "No encryption at rest".
  - `docs/dev/architecture/FEATURES.md:30-74` — fila `encryption` (`:47`).
- **Archivos leídos (secciones funcionales completas):**
  - `src/storage/engine/init.rs:1-608` — `open_with_config`, `init_storage`, `init_indexes` (`vector_index.bin` + `vstore_L*.vanta`), `recover_state` (WAL replay).
  - `src/storage/engine/mod.rs:620-749` — `create_snapshot` (Unix hardlink / Windows copy; mirror `data/` + `backend/`).
  - `src/backend.rs:1-130` — `BackendPartition` (incl. `TextIndex`, `SparseIndex`, `Versions`), `BackendWriteOp`.
  - `src/backends/fjall_backend.rs:1-120` — `FjallBackend::open` → `Database::builder(path)` (10 keyspaces, sin cipher).
  - `src/backends/registry.rs:1-110` — factory path (sin cipher).
  - `src/sdk/builder.rs:60-139` — `Embedded::open`/`open_with_config` (sin cipher).
  - `src/config.rs` (secciones) — `encryption_key` (`:225`, `:861`, `:1380-1386` env), builder `with_encryption` (`:1661`).
  - `src/lib.rs:20-79` — tabla de features (`encryption` = "AES-256-GCM at-rest encryption", `:29`).
- **Archivos referenciados hacia dentro (imports/dependencias):** `vfile.rs` importa `crate::crypto::{Cipher, EncryptionStream}` bajo `#[cfg(feature = "encryption")]` (`:16-17`); `crypto.rs` importa `crate::config::Config` (`:42`); `vanta-proxy/src/envelope.rs` importa `vantadb::crypto::Cipher`.
- **Referencias entrantes (grep `with_cipher|encryption_stream|resolve_cipher|EncryptionConfig|Cipher::from_env|derive_namespace`):** workspace completo → `crypto.rs` (definición + tests), `vfile.rs` (definición), `vanta-proxy/src/envelope.rs:119-146,198,232` (consumidor real). **`with_cipher`/`encryption_stream`/`resolve_cipher`/`EncryptionConfig`: 0 callers.** Grep `encrypt|decrypt` en `src/`: 0 usos fuera de `crypto.rs`.
- **Veredicto impacto:** **BAJO para el fix** (método feature-gated sin callers; el cambio restaura la semántica documentada y agrega un test). **ALTO para el hallazgo** (FIND-249: la feature `encryption` no protege ningún artefacto on-disk — decisión de diseño, no fix apurado). Sin cambios de formato, sin migración, sin blast radius a paths calientes.

## Mapa artefacto → cifrado (CONTRATO PRINCIPAL — evidencia por artefacto)

> Método: write path trazado por código (grep exhaustivo + lectura) + **probe runtime con tmpdir** (test temporal `tests/dur02_encryption_probe.rs`, feature `encryption`, `VANTADB_ENCRYPTION_KEY` válida, canary `DUR02_CANARY_plaintext_7f3a9b2c` en el payload). Ejecutado 2026-10-04: `cargo nextest run --profile audit -p vantadb --features encryption --test dur02_encryption_probe --no-capture` → **1 passed** (probe documenta el gap; archivo temporal eliminado tras capturar evidencia).

| # | Artefacto on-disk | Ubicación real (probe) | Write path (punto de escritura) | ¿AES-256-GCM? | Evidencia |
|---|-------------------|------------------------|--------------------------------|---------------|-----------|
| 1 | VantaFile vector store (L0-L3) | `data/vstore_L0..L3.vanta` (64 MiB c/u) | `SegmentRegistry::open_or_create` → `File::open` (`init.rs:372`); escrituras por mmap (`write_header`, payloads) | ❌ **NO** | `File.cipher` = `None` por construcción (`vfile.rs:174,243`); `init.rs` nunca llama `with_cipher`; cipher solo se setea vía `with_cipher` (0 callers) |
| 2 | HNSW index | `data/vector_index.bin` | `port_impl::open_index_port`/`new_mmap_port` (`init.rs:313-338`); `index/serialize/file.rs:25` (`File::create`) | ❌ **NO** | 0 referencias a `crypto` en `src/index/**`; sin cipher en el port ni en el serializer |
| 3 | WAL (sharded) | `data/vanta.shard0..3.wal` + `data/vanta.wal.shards` | `wal.rs:320,591` / `wal_sharded.rs:1148,1189` (`OpenOptions` append) | ❌ **NO** | 0 hits `encrypt` en `wal*.rs`; `encryption_stream()` (el único puente previsto) nunca se invoca; **probe: canary encontrado en `data/vanta.shard0.wal`** |
| 4 | Backend KV (Fjall default) | `0.jnl`, `keyspaces/0..10/**`, `version` (raíz del storage) | `FjallBackend::open` → `Database::builder(path)` (`fjall_backend.rs:56-59`) | ❌ **NO** | Sin cipher (grep `encrypt` en `backends/` = 0); **probe: canary encontrado en `0.jnl`** |
| 4b | ↳ partición `text_index` (postings BM25) | keyspace del backend | `text_index.rs:755-842` + `sdk/api/memory.rs:1473,1510` → `BackendWriteOp{partition: TextIndex}` → `backend.batch_write` | ❌ **NO** | Persistencia = KV backend (#4); postings en claro |
| 4c | ↳ particiones `sparse_index`/`versions`/`namespace_index`/`payload_index` | keyspace del backend | mismas ops (`backend.rs:34-56`) | ❌ **NO** | Mismo KV backend |
| 5 | edge_index | **no existe artefacto** | In-memory `DashSet<(u128,u128)>` (`edge_index.rs:7-9`); reconstruido desde metadata del backend al abrir (`init.rs:142` comenta el patrón) | **n/a (derivado)** | 63L, sin `File`/serialización; su sustrato on-disk son #3/#4 |
| 6 | Snapshots | `data/snapshots/<name>/{data,backend}/**` | `engine/mod.rs:647-724` — mirror de `data_dir` + backend (hardlink Unix / copy Windows) | ❌ **NO** | Espejo byte-a-byte de #1-#4: hereda el estado (plaintext) del origen |
| 7 | Export/backup manual (`vanta-cli backup/export`) | fuera del data dir | `cli_handlers/*` | ❌ NO (fuera de scope estricto) | No aplica cifrado tampoco; mencionado en FIND-249 |

> **Nota (review P2-01, H1):** hay artefactos payload-bearing adicionales que comparten los mismos write paths sin cifrar y no figuran como filas propias: WAL archivado `vanta.wal.<ts>` (`wal.rs:542-552`), temp de compactación `vstore_L*.vanta.tmp` (`storage/archive.rs:76-92`), WAL corrupto renombrado (`wal.rs:770-773`); metadata-only: `.vanta.lock` (`init.rs:168`), `.vanta.schema` (`schema.rs:172`). Ninguno cambia el veredicto (0/N plaintext); foldados a FIND-249.

**Veredicto del contrato:** **0/6 artefactos cifrados.** Con la feature `encryption` activa y `VANTADB_ENCRYPTION_KEY` válida, todos los artefactos quedan **plaintext**. La feature compila (`cargo check -p vantadb --features encryption` ✅, 34s) pero no está cableada a ningún write path. Lo que **sí** funciona: las primitivas (`Cipher`, `EncryptionStream`, KDF, HKDF por namespace) y su único consumidor real — el envelope AEAD de `vanta-proxy` (VER-03), que cifra los turnos capturados con `derive_namespace`.

**Qué protege AES-256-GCM (para clasificar bien los gaps):** confidencialidad **e** integridad (AEAD) del payload cifrado — `aes-gcm 0.11.1`, auditado por NCC Group, constant-time ([docs.rs/aes-gcm](https://docs.rs/aes-gcm/latest/aes_gcm/), verificado 2026-10-04). **Qué NO protege:** metadatos — nombres/tamaños de archivo, offsets, layout, patrones de acceso y número de registros siguen visibles aunque el payload se cifre. Por eso un futuro wiring debe declarar su alcance (payload-only) y no prometer "confidencialidad total".

## Contrato

"Mapa artefacto→cifrado (WAL, text_index, HNSW, edge_index, snapshots) con evidencia de código **y** probe runtime tmpdir + gaps clasificados (fix / FIND / wontfix documentado); gap accionable pequeño resuelto: `File::encryption_stream` usa el cipher adjunto (`with_cipher`) con test RED→GREEN; `cargo fmt --check` + `cargo clippy -p vantadb --features encryption --all-targets -- -D warnings` + `cargo nextest run --profile audit -p vantadb --features encryption -E 'test(/vfile|encryption/)'` verdes; gates docs (check-links/check-docs) 0."

## Spec (SDD — decisiones por evidencia)

> La tarea no agrega símbolos públicos nuevos (el fix corrige el cuerpo de un método existente feature-gated; no hay endpoint/tool/firma nueva). Decisiones de clasificación y fix, resueltas por evidencia:

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | ¿Cablear AES a los write paths ahora? | A) Fix apurado en WAL/vfile/HNSW (riesgo alto: mmap + formato on-disk + replay; toca `wal.rs` prohibido por DUR-01) / B) **FIND + decisión de diseño** (pre-mortem #2 del plan: "hallazgo grande → FIND + decisión, no fix apurado") | ✅ **B** — evidencia: 0/6 artefactos cifrados → es diseño, no bug local (FIND-249) |
| 2 | ¿Qué gap pequeño SÍ se arregla? | A) `encryption_stream()` resuelve `Cipher::from_env()` e ignora `self.cipher` (contradice su doc y el setter `with_cipher`; bug de la API) / B) dejarlo | ✅ **A** — fix 1 línea + test (código muerto hoy, correcto para cuando se cablee) |
| 3 | ¿Docs que afirman cifrado at-rest? | A) Corregir los 2 claims inexactos (CONFIGURATION.md:73, FEATURES.md:47) / B) FIND doc-sweep | ✅ **A** — fix docs pequeño; `HTTP_API.md:701` ya es honesto (no tocar) |
| 4 | ¿`edge_index` = gap? | A) FIND por "sin cifrar" / B) OK-justificado: no tiene artefacto propio (derivado en memoria; su sustrato son #3/#4) | ✅ **B** — evidencia `edge_index.rs:7-9` + `init.rs:142` |
| 5 | ¿`Cipher::from_env()` leyendo `Config::default()` (no la config del engine)? | A) Cambiar a config explícita / B) registrar como nota en FIND (la env var es la fuente documentada; `Config::default()` la lee en `config.rs:1380-1386`) | ✅ **B** — sin callers de storage hoy; no ampliar scope |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. `crypto.rs` (primitivas, framing, KDF, tests) no cambia — el fix es en `vfile.rs`.
  2. `vanta-proxy/src/envelope.rs` (consumidor real de `Cipher`) no cambia — VER-03 intacto.
  3. Ningún artefacto on-disk cambia de formato ni de comportamiento en esta tarea (el fix es a código sin callers; el cifrado sigue sin aplicarse — FIND-249).
  4. `wal.rs` y `sdk/api/memory.rs` NO se tocan (WIP de DUR-01/DUR-03).
  5. La feature `encryption` sigue siendo opt-in y compilable (`--features encryption`).
- **Comandos de verificación:** `cargo check -p vantadb --features encryption` · `cargo nextest run --profile audit -p vantadb --features encryption -E 'test(/vfile|encryption/)'` · `cargo nextest run --profile audit -p vantadb -E 'test(/vfile/)'` (sin feature) · `cargo fmt --check` · `cargo clippy -p vantadb --features encryption --all-targets -- -D warnings` · `node scripts/docs/check-links.mjs` + `check-docs.mjs`.
- **Deuda pendiente:** FIND-249 (cablear cifrado o ajustar feature a "primitivas only" — decisión de diseño con el owner/arch). Nota: `Cipher::from_env` vía `Config::default()` (Spec #5).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** sin deuda nueva (fix de 1 línea + test + docs; FIND-249 documenta deuda pre-existente, no la introduce).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Mapa completo con evidencia por artefacto + probe runtime + clasificación de gaps + fix+test + docs corregidos + fmt/clippy/nextest scoped verdes |
| **Commit** | Commit atómico conventional `fix(security):` + `git diff` limpio + verificación mecánica (nunca auto-reporte) |
| **Release** | n/a (plan: "release = n/a"). `dev-tools/verify.ps1` completo queda para el batch de cierre del plan (push diferido por instrucción del owner) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius crypto/vfile; usado) + grep exhaustivo (0 callers — verificado) + `codebase-memory-mcp_check_index_coverage`
- `cargo check -p vantadb --features encryption` (activabilidad) + probe `cargo nextest ... --features encryption --test dur02_encryption_probe`
- `cargo nextest run --profile audit -p vantadb --features encryption -E 'test(/vfile|encryption/)'` (loop TDD)
- `campaign_verify_cmd` (verify mecánico) + `pwsh dev-tools/ocr-review.ps1` (OCR delegation al cierre)

**Skills cargadas (SDP v3):** `security-and-hardening` (auditoría de control de seguridad) · `test-driven-development` (fix con test RED→GREEN) · `source-driven-development` (verificación de primitivas AES-GCM contra docs oficiales) · `deprecation-and-migration` (pinned storage/schema — aplicado: clasificación "wiring vs primitivas-only", precedente PITR/FIND-26) · `documentation-skill` (edits en `docs/**`) · `coordinated-web-search` (cascada de fuentes) · base auto (campaign-executor/progreso/ponytail). SDP: 7 cargadas; `incremental-implementation`/`context-engineering` aplicadas como práctica (no re-cargadas); `writing-*` excluidas por no aplicar.

## Investigation Notes

- **Evidencia de código (por artefacto):** ver §Mapa. Puntos de no-cifrado: `init.rs:313-338,372` (HNSW/VantaFile), `wal.rs:320,591` + `wal_sharded.rs:1148,1189` (WAL), `fjall_backend.rs:56-59` (backend KV), `engine/mod.rs:678-679` (snapshot mirror), `edge_index.rs:7-9` (derivado).
- **Probe runtime (tmpdir):** artefactos creados por un `put` (sin vector): `data/vanta.shard0..3.wal` (+`.shards`), `data/vector_index.bin`, `data/vstore_L0..L3.vanta`, backend Fjall (`0.jnl`, `keyspaces/0..10/**`, `version`, `lock`). Canary plaintext encontrado en **`data/vanta.shard0.wal` (378B)** y **`0.jnl` (journal Fjall)**. `Cipher::from_env()` resolvió la key y el roundtrip de las primitivas fue OK en el mismo proceso → el gap es de wiring, no de primitivas.
- **Web (docs oficiales, verificado 2026-10-04):** [aes-gcm 0.11.1 — docs.rs](https://docs.rs/aes-gcm/latest/aes_gcm/) — "Authenticated Encryption with Associated Data (AEAD) cipher"; auditado por NCC Group; constant-time (AES-NI/CLMUL). Cita usada para: (a) el módulo usa la primitiva correctamente; (b) el alcance AEAD es payload (confidencialidad+integridad), no metadatos.
- **Precedentes internos:** `HTTP_API.md:701` ya declara "No encryption at rest... block-storage encryption is the operator's responsibility" (honesto — se mantiene). `FEATURES.md:58` documenta la remoción de `pitr` por "dead code sin wiring desde el engine (FIND-26)" — precedente de cómo se resuelve una feature sin cablear (aunque aquí las primitivas tienen consumidor real: el proxy).
- **Por qué no se cableó:** hipótesis (no verificada, sin evidencia): TSK-72 entregó primitivas (fase 5, `b78a9b5a`); el wiring a VantaFile/HNSW mmap requería decisión de layout (mmap no soporta cifrado transparente sin página/offset mapping). La evidencia disponible solo confirma el estado actual.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — (a) cobertura real determinada (grep 0 callers + probe runtime); (b) clasificación de gaps decidida (FIND-249 / fix pequeño / OK-justificado); (c) método de verificación alternativo validado (feature compila + probe tmpdir) |
| Pendientes de ejecución (downhill) | 4 steps (3-6) |
| % completado | 40% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — es el objeto de la tarea. Hallazgos: (1) control "encryption at-rest" inexistente en la práctica → FIND-249 con severidad alta; (2) docs que sobre-prometían → corregidos; (3) el fix de `encryption_stream` cierra una inconsistencia de la API de seguridad (cipher adjunto ignorado). Sin input de usuario nuevo, sin auth, sin dependencias nuevas. Trust boundary: archivos on-disk (at-rest).
- [x] **PERFORMANCE** — no aplica: no hay hot path tocado (código feature-gated sin callers + docs). Regla 9 no dispara (no es optimización).

## Steps

### Step 1 — Probe runtime tmpdir (evidencia empírica del gap)

- **Archivos:** `tests/dur02_encryption_probe.rs` (temporal, eliminado tras capturar)
- **Acción:** con feature `encryption` + `VANTADB_ENCRYPTION_KEY` válida: verificar primitivas (roundtrip) + `put` con canary + inventario de artefactos + scan de canary en todos los archivos.
- **Verify:** `cargo nextest run --profile audit -p vantadb --features encryption --test dur02_encryption_probe --no-capture` → passed + hits reportados
- **Evidencia:** ✅ `cargo check -p vantadb --features encryption` (34s) + probe 1 passed (15s): canary en `data/vanta.shard0.wal` y `0.jnl`; inventario completo transcripto en §Investigation Notes. Archivo temporal eliminado (`git status` limpio de tests nuevos).
- **Estado:** ✅ COMPLETED

### Step 2 — Task file + FIND-249 en Backlog

- **Archivos:** `docs/dev/tasks/DUR-02.md`, `docs/dev/Backlog.md`
- **Acción:** escribir este task file (formato canónico) + fila `FIND-249` (10 columnas, esquema `backlog-format.md`) con el hallazgo, evidencia y acción propuesta.
- **Verify:** `node scripts/docs/check-docs.mjs` (kind/frontmatter) + fila parseable
- **Evidencia:** ✅ (este archivo + fila FIND-249 en Backlog; evidencia del probe transcripta)
- **Estado:** ✅ COMPLETED

### Step 3 — Fix `encryption_stream` (RED→GREEN)

- **Archivos:** `src/storage/vfile.rs`
- **Acción:** test `test_vfile_encryption_stream_uses_attached_cipher` (feature-gated): el stream debe usar el cipher adjunto (`with_cipher`) — verificado por (a) decrypt del frame crudo con el cipher adjunto, (b) ausencia de plaintext on-disk, (c) roundtrip por el stream. Correr ANTES del fix (RED: `encryption_stream()` resuelve `Cipher::from_env()` → `None` sin env / cipher distinto). Fix: `let cipher = self.cipher.clone()?;`.
- **Verify:** `cargo nextest run --profile audit -p vantadb --features encryption -E 'test(/vfile_encryption_stream/)'` → RED luego GREEN
- **Evidencia:** ✅ RED verificado en HEAD pre-fix: FAILED — `an attached cipher must yield an encryption stream` (`encryption_stream` devolvía `None` por resolver `Cipher::from_env()` sin env). Fix (`self.cipher.clone()?`) → GREEN 1/1 + filtro amplio `test(/vfile/) + test(/encryption/)` → **42/42 passed**.
- **Estado:** ✅ COMPLETED

### Step 4 — Docs: claims ajustados a la realidad

- **Archivos:** `docs/user/operations/CONFIGURATION.md` (fila `encryption_key`), `docs/dev/architecture/FEATURES.md` (fila `encryption`)
- **Acción:** declarar que la feature entrega primitivas pero **no** cifra los artefactos de storage hoy (FIND-249); consumidor actual = envelope del proxy. `HTTP_API.md` ya es honesto — no se toca.
- **Verify:** `node scripts/docs/check-links.mjs` + `check-docs.mjs` exit 0
- **Evidencia:** ✅ `CONFIGURATION.md:73` + `FEATURES.md:47` corregidos (cifrado no aplicado a storage; consumidor actual = envelope del proxy); `FIND-249` en Backlog (fila 10-col tras FIND-248); `gen-index --write` regeneró `docs/index.md`/`llms.txt` (+DUR-01/DUR-02; contadores 1497→1499 y 1049→1051); `check-links`/`check-docs`/`gen-index --check` exit 0.
- **Estado:** ✅ COMPLETED

### Step 5 — Verify full scoped + OCR delegation

- **Archivos:** —
- **Acción:** `cargo fmt --check` + `cargo clippy -p vantadb --features encryption --all-targets -- -D warnings` + nextest scoped (con y sin feature) + gates docs; OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) con Rule Groups aplicados.
- **Verify:** todos exit 0 / sin Critical/High
- **Evidencia:** ✅ `cargo fmt -p vantadb --check` exit 0 · `cargo clippy -p vantadb --features encryption --all-targets -- -D warnings` exit 0 · nextest scoped 42/42 (con feature) · `check-links`/`check-docs`/`gen-index --check`/`validate-docs-coverage.ps1` (0 gaps) exit 0 · OCR delegation aplicada (Rule Group 2 → `vfile.rs`): **0 Critical/High** (clone necesario — `EncryptionStream` posee el `Cipher` por valor; sin unwrap/unsafe nuevos en producción) · review P2-01 `vanta-review` ✅ APPROVE (H2/H3/H4 incorporados; H1 foldado; H5 opcional diferido).
- **Estado:** ✅ COMPLETED

### Step 6 — Review P2-01 + commit local

- **Archivos:** `docs/dev/tasks/DUR-02.md`
- **Acción:** review adversarial por agente distinto (tier **Adversarial**: `src/storage/**` matchea HARD-02 → fork a `vanta-review`); registrar veredicto en §Review; commit **LOCAL** `fix(security): DUR-02 — ...` solo con archivos propios (nunca push).
- **Verify:** `git show --stat HEAD` limitado a archivos propios; veredicto en §Review
- **Estado:** ⬜ PENDING

## Dependencias

- F0 — sin dependencias (el plan declara "F0 — sin dependencias").
- nextTask: BENCH-01.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Tier:** **Adversarial** (paths del diff: `src/storage/**` matchea la tabla HARD-02 → review adversarial obligatorio por `vanta-review`).
- **Revisor:** `vanta-review` (contexto fresco — sesión `ses_ef9ba96b4ffe2yjtiCxKVTcxaF`, no participó de la implementación) · 2026-10-04.
- **Enfoque:** fix mínimo vs rediseño (`EncryptionStream<&Cipher>` — descartado: 0 callers + superficie pública sin beneficio); clasificación FIND vs fix apurado (correcta per pre-mortem #2 del plan); docs verificados contra el consumidor real (`envelope.rs:125`); mapa re-verificado artefacto por artefacto.
- **Cómo se probó:** re-ejecutó `nextest --features encryption` (test nuevo 1/1; filtro amplio 42/42) · verificó `git show HEAD:src/storage/vfile.rs:461` (bug pre-fix) · grep independiente de 0 callers (métodos inherentes — sin dispatch dinámico) · grep `encrypt|cipher` en `wal*`/`backends`/`index` = 0 hits · gates docs exit 0.
- **Checklist anti-hábitos tóxicos:** ✅ todos ok — sin salidas inventadas (outputs re-ejecutados por el revisor); sin done sin verificar; fallos parciales reportados dentro de presupuesto (check-links 44/58 y wikilinks 30/40 budgeted, ninguno nuevo); evidencia code-corroborada para el probe no re-ejecutable; chequeo de seguridad no degradado (semántica restaurada + gap escalado).
- **Hallazgos:** H1 Low (artefactos payload-bearing omitidos → foldado al mapa/FIND-249) · H2 Low ("9 gates" → 10) · H3 Nit (`lib.rs:53` → `:57`) — incorporados; H4 proceso (sync de Steps/§Review — este bloque) · H5 Optional (probe transcript-only; reproducibilidad futura vía `#[ignore]`/script — diferido, documentado).
- **Veredicto:** ✅ **approve** (0 Critical / 0 High / 0 Required).

## Notas

- **Hallazgo principal (FIND-249):** la feature `encryption` no cifra **ningún** artefacto on-disk (0/6); es un control de seguridad inexistente en la práctica. Decisión requerida: cablear (diseño: mmap VantaFile/HNSW + WAL frame-aware + wrapper KV) o declarar "primitivas only" y sacarla de claims/candidatas Pro hasta que exista wiring.
- **Fix aplicado (pequeño):** `File::encryption_stream` ignoraba el cipher adjunto y resolvía `Cipher::from_env()` — contradecía su propio doc y el setter `with_cipher`; corregido + test.
- **Prohibiciones respetadas:** `wal.rs` solo lectura (DUR-01), `sdk/api/memory.rs` no tocado (DUR-03), plan file/`opencode.jsonc` no tocados.
- **Review P2-01:** H5 opcional diferido (probe transcript-only; futura reproducibilidad vía test `#[ignore]`/script — no bloquea; H1-H4 incorporados).
- WIP ajeno en el árbol (no stage): archivos de DUR-01/DUR-03/otros workers.

## RESULTADO §7 (contrato de retorno — pipeline-full)

```
RESULTADO: ⬜ (se completa al cierre)
```
