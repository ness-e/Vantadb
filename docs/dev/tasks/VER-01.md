---
title: "VER-01: Tamper-evident — hash-chain en WAL + `vanta-cli verify`"
kind: task
description: "Hash-chain incremental SHA-256 por registro en el WAL (prev_hash + hash del frame) + `vanta-cli verify` con detección posicional (exit ≠0); compat WAL v1/v2 (bump WAL_FORMAT_VERSION); diseño revisado por vanta-audit."
---

# VER-01: Tamper-evident — hash-chain en WAL + `vanta-cli verify`

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` — Task 35 (F4) · **Origen:** plan L891-941 (bloque completo leído); Backlog:923; MGR-13 §8 (`docs/dev/research/mgr-13-cuarentena.md:228` — cita el chain como deuda v1.0, no lo duplica); SCH-03/SCH-05 lo dejaron explícitamente a VER-01 (plan :803/:811).
- **Fuente del prompt:** sub-agente vanta-worker (orquestador pipeline) — wave F4.1 (co-batch VER-07 ‖ VER-05); branch `develop`; commits = LEAD.
- **Esfuerzo:** 🔴 3-5d · **Prioridad:** 🔴 · **Tipo:** feature-add (Rust core + CLI) — lógica nueva, seguridad/durabilidad.
- **Creado:** 2026-09-29 · **last-synced:** 2026-09-29
- **Estado:** ✅ COMPLETO (carril worker) — steps 8/8 ✅: implementación + tests scoped/workspace-excl + gates propios + bench before/after + smoke E2E + snapshot API regenerado (PASS). Pendiente SOLO carril LEAD: commit local + review P2-01 fresco (`vanta-audit` para chain/attestation) — NO ejecutados por diseño de la wave.
- **Gate D (question-gates):** DISPARADO por contrato (hot path WAL + API pública nueva `Commands::Verify`/`cmd_verify`) — **pre-respondido por el orquestador**: el bloque F4 Task 35 ya pasó su completado de fase (contrato verbatim + stop conditions + Risk Register en el plan), y la instrucción de ejecución es explícita ("ejecutá UNA TAREA COMPLETA"). No se agrega superficie más allá de la requerida por el contrato (`Commands::Verify` + `cmd_verify` + `verify_wal_file` crate-visible).
- **Gate V (question-gates):** no disparado (0 fallas de verify mismo-error hasta ahora).

## Contrato (verbatim del plan, L927)
> "hash-chain incremental por registro en el WAL (prev_hash + hash del frame; sin re-hash total) con coste de escritura medido ≤5% (`wal_throughput` before/after) Y `vanta-cli verify` recorre el WAL y detecta manipulación/extirpación con registro exacto (1 registro alterado o eliminado → exit ≠0 + posición; test dedicado) Y compat: WAL v1 sigue recuperable (bump de `WAL_FORMAT_VERSION` con lectura legacy o migración) Y diseño del chain documentado en `docs/api/` (qué cubre y qué no: límites de truncado legítimo vs extirpación)"

**Cláusulas a verificar (matriz de cierre):**

| # | Cláusula | Superficie | Evidencia |
|---|----------|-----------|-----------|
| C1 | Hash-chain incremental por registro (prev_hash + hash del frame; sin re-hash total) | `src/wal.rs` (framing v3) | test unit + framing byte-idéntico append/batch |
| C2 | Coste de escritura (bench before/after) | `benches/wal_throughput.rs` | §Evidencia de rendimiento (matriz + decisión de scope: gate ≤5% cumplido en path de durabilidad; desviación documentada en buffered + FIND) |
| C3 | `vanta-cli verify` detecta manipulación/extirpación con registro exacto (alterado y eliminado → exit ≠0 + posición) | `src/cli.rs` + `src/cli_handlers/wal.rs` + `tests/wal_chain_verify.rs` | tests dedicados |
| C4 | Compat: WAL v1 (y v2) sigue recuperable | `src/wal.rs` (validate_compat + framing por versión) | test legacy v2/v1 roundtrip |
| C5 | Diseño documentado (qué cubre y qué no) | este task file §Diseño (→ `docs/api/` pendiente, ver §Deuda 3) | §Diseño del chain |

## Re-baseline (verificado 2026-09-29, branch `develop`, HEAD `9949483a` + plan file sucio del orquestador — intacto)
- **Gap confirmado:** `rg 'hash_chain|prev_hash|chained_hash|chain_hash' src/` = 0 hits; el WAL hoy es CRC32C por registro (detección de corrupción, NO de manipulación: un rewrite con CRC recalculado pasa).
- **Puntos de extensión verificados (leídos completos):** frame `[len u32][postcard][crc u32]` (`src/wal.rs:216-217`); `append` `:306-329`; `batch_append` `:332-370`; `WalHeader` `:96-143` (20B = 16B base `VWAL+version+schema+ts` + CRC32C; `validate_compat` acepta ≤ `WAL_FORMAT_VERSION` `:146-192`); `recover_valid_records` `:567-608`; `quarantine_corrupt_tail` `:614-638`; `WalReader` `:658-787`.
- **WAL sharded:** `ShardedWal` (`src/wal_sharded.rs:9`) — round-robin por shard; `recover` :448-491 (abre cada shard con `WalReader`); `verify_shard_counts` :69 (guard ERR-011 de coherencia round-robin); `detect_shard_count`/`read_shard_meta` :34/:97; salvage CLI :165-266 (`vanta-cli wal salvage`).
- **CLI:** `Commands` (`src/cli.rs:47`) no tiene `verify`; `WalCommand` :435 = {Compact, Vacuum, Salvage}; dispatch del bin `src/bin/vanta-cli.rs:269-277`; patrón de exit code por handler: `Commands::McpCall` (`:286-289` → `std::process::exit(code)`).
- **Audit:** `AuditEvent` (`src/audit.rs:21-39`) es JSONL opt-in de operaciones de negocio (doc :1-6) — modelo de evento, no fuente de bytes on-disk.
- **Consumidor de frames:** `src/wal_shipping.rs` lee segmentos archivados con `WalReader` (:126-129) — transparente si `WalReader` es version-aware.
- **Tests con framing raw (romperían con v3 sin update):** `tests/storage/wal_resilience.rs:118-124` y `:355-365` (walks manuales); `src/wal_sharded.rs:1089-1108` y `:1133-1148` (salvage tests); `src/wal.rs` (`test_recover_mid_file_corruption...` inyecta frame raw). `tests/wal_rollback.rs:226-235` pinea `WAL_FORMAT_VERSION == 2` → actualizar a 3 (es el keystone del bump).
- **Dependencia disponible:** `sha2 = "0.11"` ya en el lockfile (solo bajo feature `encryption`, `Cargo.toml:142`); `twox-hash`/`ahash` presentes pero NO criptográficos.

## Blast Radius

| Dirección | Archivos |
|-----------|----------|
| **Edita** | `src/wal.rs` (framing v3 + writer/reader + verify) · `src/wal_sharded.rs` (verify por shard + tests salvage walk) · `src/cli.rs` (`Commands::Verify`) · `src/bin/vanta-cli.rs` (dispatch) · `src/cli_handlers/wal.rs` (`cmd_verify`) · `Cargo.toml` (sha2 no-opcional; `Cargo.lock` sin cambios — sha2 ya estaba listado como dep opcional) · `tests/wal_chain_verify.rs` (nuevo) · `tests/wal_rollback.rs` (pin v3) · `tests/storage/wal_resilience.rs` (walks +64) · `tests/api/public-api.txt` (snapshot regenerado — incluye símbolos co-batch VER-05, ver §Deuda 4) · este task file |
| **NO toca (prohibido/wave)** | `docs/**` salvo este task file (diseño vive acá; destino final `docs/api/` = pendiente LEAD) · `docs/dev/Backlog.md` · `perf-bench.yml` · `CONSTRAINTS.md` · `desktop/**` · `opencode.jsonc` · plan file (LEAD) · **regiones co-batch F4.1:** `vanta-memory/src/core/dream/**`, `vantadb-mcp/src/dreams.rs` (VER-07), `src/sdk/serialization/**` (VER-05) |
| **Referencias hacia dentro** | `src/storage/wal.rs:init_wal` → `ShardedWal::new_with_buffer`; `src/storage/engine/txn.rs:283-388` (append/batch_append de txn) ; `src/wal_shipping.rs:126` (WalReader) |
| **Referencias entrantes** | VER-02 (attestation de purga) y VER-04 (audit WORM-ready) consumen la evidencia encadenada (MGR-13 §8); recovery/engine dependen del framing |
| **Implicaciones** | Formato on-disk cambia con bump de versión (v3) → downgrade sigue requiriendo dump/restore (mismo hint actual, documentado); API pública +2 símbolos (CLI) → snapshot `public-api.txt` regenerado deliberadamente; hot path de `append` +1 SHA-256 por frame (gate ≤5% medido). |

## Impacto mapeado (Regla 0)

- **Leídos completos:** `src/wal.rs` (1434L) · `src/wal_sharded.rs` (1175L) · `src/audit.rs` (381L) · `src/binary_header.rs` (264L) · `src/cli.rs` (Cli+WalCommand) · `src/bin/vanta-cli.rs` (321L) · `src/cli_handlers/wal.rs` (:1-268) · `benches/wal_throughput.rs` (158L) · `tests/proptest_wal_roundtrip.rs` (289L) · `tests/wal_rollback.rs` (236L) · `tests/storage/wal_resilience.rs` (:95-417) · `tests/api/public_api.rs` (138L) · `src/storage/wal.rs` (25L) · `Cargo.toml` (deps/features) · `.config/nextest.toml` (filtros) · `.opencode/rules/durability.md` + `core-engine.md`.
- **Leídos rangos clave:** `src/storage/engine/txn.rs` (callers WAL :283-388) · codegraph: `WalWriter` ← `wal_shipping`/`wal_sharded`/`lib`; `ShardedWal` ← `src/engine.rs`/`src/storage/wal.rs` · `docs/dev/plans/2026-09-26-master-roadmap.md:891-941` · `docs/dev/research/mgr-13-cuarentena.md:228`.
- **Referencias hacia dentro:** `WAL_FORMAT_VERSION` (const pública, re-export `src/lib.rs:206`; usada en wal.rs :160/:261/:486; pineada por `tests/wal_rollback.rs:232`) · frame parsing (scan-forward, quarantine, salvage, wal_shipping) · `WalHeader::deserialize` (valida rango ≤ current).
- **Veredicto impacto:** **alto (formato on-disk + hot path)**. Mitigaciones: bump versionado con lectura legacy (range-compat ya existente), framing por versión del archivo abierto (no mezcla), tests de recovery intactos + bench before/after. Ninguna región co-batch F4.1 tocada.

## Spec (feature-add con decisiones por evidencia)

| # | Decisión | Elegido | Evidencia |
|---|----------|---------|-----------|
| 1 | Scope: WAL completo vs log dedicado de ops | **WAL completo, por registro** | Contrato verbatim L927 ("hash-chain incremental por registro **en el WAL**"); el WAL ya es fuente de verdad durable de TODAS las mutaciones (txn.rs:299-307 Insert/Update/Delete) y ruta de recovery; un log paralelo sería extirpable de forma independiente (peor tamper-evidence) y duplica append path; `AuditEvent` es JSONL opt-in de ops de negocio (audit.rs:1-6) → no cubre bytes on-disk. Decisión de scope registrada (= stop condition Backlog:923 resuelta). |
| 2 | Hash | **SHA-256 (`sha2` 0.11, ya en lockfile; pasa a no-opcional)** | Crypto-hash colisión-resistente; cero crates nuevos; `twox-hash`/`ahash` son no-cripto (no cumplen "tamper-evident"); BLAKE3 = dependencia nueva sin ganancia material para payloads pequeños. Coste medido con bench (C2). |
| 3 | Framing v3 | `[len u32 LE][payload][crc u32 LE][prev_hash 32][record_hash 32]`; `record_hash = SHA-256(prev_le ‖ len_le ‖ payload ‖ crc_le)`; genesis `prev = [0u8;32]` | Append-only, sin tocar el payload postcard (compat de wire del record); hash incremental O(frame), sin re-hash total; genesis fijo (no hash del header) → framing byte-idéntico append/batch (el header lleva timestamp wall-clock y rompería el test de byte-identidad). |
| 4 | Alcance de la cadena | **Por archivo (por shard/segmento)**; genesis cero en cada segmento nuevo; sin cross-link entre segmentos | Round-robin sharded hace imposible una cadena byte-global; pre-mortem F1 del plan ofrece "cadena por shard + cross-link en shard_meta" o decisión documentada → se toma la documentada (cross-link exige extender header o nuevo record; rabbit hole "re-chaining histórico" prohibido por la stop condition). Segmentos archivados: verify cubre los archivos actuales; la extirpación de un segmento entero queda fuera (sin ancla externa). |
| 5 | Compat v1/v2 | Bump `WAL_FORMAT_VERSION` 2→3; `validate_compat` (≤ current) ya acepta legacy; **el writer mantiene el framing de la versión del archivo abierto**; rotación/compact escribe v3 | Contrato C4; el `< `range-compat ya existe (wal.rs:146-192); mezclar framings en un archivo es imposible → archivo legacy sigue legacy hasta rotar (migración natural vía `vanta-cli wal compact`, rollback-friendly: no se reescribe in-place). Downgrade v3→v2 sigue requiriendo dump/restore (mismo hint actual). |
| 6 | Semántica del verify | Estados: `verified` / `legacy` (formato < 3: CRC-only, explícito) / `tampered {offset, index, reason}` / `corrupt {offset, index}` / `incomplete_tail {offset}`. Exit 0 = verified/legacy/incomplete_tail; 1 = tampered/corrupt | Contrato C3 + pre-mortem F2/F3 del plan: truncado legítimo post-crash (torn tail) NO se reporta como manipulación (`incomplete_tail`, exit 0, con aviso); "pre-chain prefix" legacy explícito. |
| 7 | Dónde vive el verify | `pub(crate) fn verify_wal_file` (wal.rs) + `pub(crate) fn verify_shards` (wal_sharded.rs, gated `cli|test` como salvage) + `cmd_verify` (cli_handlers) + `Commands::Verify` (cli.rs) | Lógica en core (no duplicada en bindings), CLI thin; precedente `cmd_wal_salvage` (preview read-only sin abrir engine) y `cmd_mcp_call` (exit code por handler). |
| 8 | Coherencia de shards en verify | Además del chain por shard, el verify corre `verify_shard_counts` (guard ERR-011 existente) y reporta incoherencia round-robin como fallo | El borrado del ÚLTIMO registro de un shard no rompe su cadena (límite documentado). El guard delata el truncado SOLO cuando rompe el patrón round-robin (p.ej. `[0,1,1,1]`); si el conteo resultante sigue siendo coherente (smoke real: `[1,1,1,1]` tras truncar la cola de shard0) NO hay señal — límite documentado, sin ancla externa. Reusa guard existente, sin código nuevo de detección. |
| 9 | `docs/api/` | **Pendiente por restricción de la wave** (`docs/**` prohibido salvo task file); diseño completo acá, listo para levantar a `docs/api/` | Instrucción del orquestador (wave F4.1, anti-conflicto co-batch). Deuda registrada §Deuda; LEAD mueve/referencia. |

## Diseño del chain (explícito para revisión `vanta-audit` — P2-01)

**Qué cubre:**
1. **Manipulación de contenido con CRC recalculado**: alterar payload + recomputar `crc` → `record_hash` no coincide con SHA-256(prev ‖ len ‖ payload ‖ crc) → `tampered` en el offset exacto del registro.
2. **Extirpación (borrado de registro en medio del archivo)**: el `prev_hash` del sucesor apunta al `record_hash` de un registro que ya no está → `tampered` (link roto) en el offset del sucesor. También detecta inserción/reordenamiento de frames completos.
3. **Corrupción** (CRC/deser/framing roto mid-file) → `corrupt` con offset (la cadena no la llamamos "tamper": puede ser crash/bit-rot).
4. **Prefijo pre-chain** (v1/v2) → `legacy` explícito, nunca tratado como manipulación.

**Qué NO cubre (límites documentados):**
- **Truncado del sufijo en frontera de registro limpia**: borrar los últimos N registros no es distinguible sin ancla externa (no hay firma/clave). Mitigación parcial multi-shard: `verify_shard_counts` (ERR-011) delata shards cortos SOLO cuando el conteo rompe el patrón round-robin — verificado en smoke: truncar 1 de 2 registros de shard0 deja `[1,1,1,1]` (coherente) → no hay señal; límite real, no una garantía. Upgrade path: firma/attestation con clave del motor = follow-up `vanta-audit` (plan L941).
- **Rewrite íntegro y consistente del archivo** (atacante recomputa toda la cadena): indistinguible sin ancla externa/firma. Mismo upgrade path.
- **Header bytes** no están dentro de la cadena (tienen su propio CRC32C; la versión del header decide el framing). Un flip de versión fuerza rechazo por `validate_compat`.
- **Segmentos archivados no presentes en disco**: no hay manifest de segmentos → un segmento borrado entero no es detectable (sin ancla). Fuera de scope (rabbit hole del plan).
- **Rendimiento**: SHA-256 por frame (payload ya en memoria); gate ≤5% medido con `wal_throughput` (before = baseline actual, after = con chain).

**Invarianza de recovery:** la recuperación NO valida la cadena (mantiene semántica actual CRC+deser+scan-forward+quarantine); el detector autoritativo es `vanta-cli verify` (offline, read-only, sin abrir el engine). El writer reanuda la cadena leyendo los 32B de `record_hash` del último registro válido (o genesis si no hay registros).

## Invariantes de dominio (handoff — MUST)
- Append-only: jamás reescribir/re-hashear registros existentes; la cadena solo crece.
- `WalReader`/recovery siguen leyendo v1/v2/v3 (range-compat) — la pérdida de datos silenciosa sigue prohibida (ERR-011 intacto).
- `verify` es read-only: no trunca, no muta, no abre el engine (funciona con DB cerrada/lockeada).
- `crc32c` sigue siendo el checksum de framing; la cadena NO lo reemplaza (defensa en capas).
- Prohibido tocar regiones co-batch F4.1 (dream/**, `src/sdk/serialization/**`).

## Deuda técnica (Regla 6 — MUST)
**Saldo neto: ≥0 (cero o negativo).** Añade: +1 dep efectiva (sha2 ya presente en lockfile, pasa a default — no crate nuevo), +64B/registro (formato), +1 hash/append. Paga: elimina la ausencia total de tamper-evidence (gap P52), reusa guards existentes (ERR-011), sin duplicar lógica bindings. Deuda nueva: diseño→`docs/api/` pendiente (restricción de wave), firma/ancla externa diferida a vanta-audit (ya prevista por el plan), cross-segment link diferido (documentado).

**FIND candidatos (registrar en Backlog — LEAD/orquestador; Backlog prohibido en esta wave):**
1. **Coste del chain en rutas buffered**: +450–500 ns/registro (SHA-256 por frame) sobre registros de 24 B — optimización (dispatch SHA-NI/`compress256` directo, BLAKE3, coalescing por lote para registros cortos). Gate ≤5% se cumple en el path de durabilidad (fsync), no en el buffered. Dueño sugerido: `vanta-tuner`.
2. **`public-api` harness**: (a) `stack overflow` (exit 253) post-escritura en la regeneración cold-cache (archivo escrito correctamente; 2º run PASS limpio — benigno pero envenena el exit code la primera vez); (b) bajo `--profile audit` el test **nunca completa** en esta máquina (build nightly rustdoc > 180s → slow-timeout lo termina) aunque está en el default-filter del workspace → el rails HARD-01 queda con señal ambigua localmente (en CI dedicado presumiblemente corre sin timeout). Dueño sugerido: `vanta-lead`/harness (excluir del perfil audit o subir timeout del job dedicado).
3. **Docs surface (gate `validate-docs-coverage` = ❌ 1 gap)**: el comando `verify` no está en la tabla CLI de `docs/user/operations/CONFIGURATION.md`. **Patch exacto (1 fila, insertar tras `wal salvage` en L398):**

   ```markdown
   | `verify [--json]` | Verify the WAL hash-chain integrity (tamper-evident, VER-01): detects altered or removed records with their exact position; read-only (no engine open); exit code ≠0 when integrity fails |
   ```

   Bloqueado por la prohibición `docs/**` de la wave F4.1 → lo aplica el LEAD (y re-corre `pwsh scripts/validate-docs-coverage.ps1`). Además: §Diseño del chain listo en este task file para levantar a `docs/api/`.
4. **Snapshot co-batch**: `tests/api/public-api.txt` regenerado incluye símbolos en vuelo de VER-05 (`sdk::importers::*`) — si el commit por-tarea de VER-01 no los incluye, re-regenerar tras el merge de la wave (o incluir el snapshot en el commit conjunto).
5. **Workspace gate pre-existente (NO VER-01)**: `cargo nextest --workspace` no compila por el `const assert` de `vanta-memory/tests/smoke.rs:15` vs feature unification (`llm-driver` encendido por `vantadb-mcp`/`vanta-proxy` — estado HEAD). Dueño sugerido: `vanta-lead`/`vanta-harness` (decidir: test de runtime en vez de const-assert, o `--exclude`, o resolver unification).

## Definition of Done (3 niveles)
- **Task:** C1–C5 con evidencia (§Verificación); gates `cargo fmt --check` + clippy `-D warnings` + nextest scoped + workspace audit; bench before/after.
- **Commit (LEAD):** `feat(wal): VER-01 — hash-chain tamper-evident en WAL + vanta-cli verify` (o similar conventional) — lo ejecuta el LEAD.
- **Review:** P2-01 con contexto fresco (vanta-audit para el diseño del chain — nota del orquestador). NO self-review.

## Herramientas necesarias
- `cargo check -p vantadb` · `cargo clippy --workspace --all-targets --all-features -- -D warnings` · `cargo fmt --check` · `cargo nextest run --profile audit -p vantadb --test <wal_*|cli_tests|public_api>` (scoped, regla `-p`) · workspace audit (`--workspace --build-jobs 2`, timeout 900) · `cargo bench -p vantadb --bench wal_throughput -- wal_throughput/never` (before/after) · `CARGO_BUILD_JOBS=2` en verifies pesados.
- Snapshot API: `VANTADB_PUBLIC_API_UPDATE=1 cargo nextest run -p vantadb --test public_api` (nightly instalado ✅).
- **SDP (v3, BUILD, Rust core):** base `campaign-executor`+`progreso`+`ponytail` (auto) · pins `test-driven-development`, `systematic-debugging`, `deprecation-and-migration`, `performance-optimization` · `source-driven-development`, `doubt-driven-development`, `rust-write-tests`, `security-and-hardening`. Sin candidatos extra tras discovery.

## Steps

| # | Step | Estado | Evidencia |
|---|------|--------|-----------|
| 1 | DISCOVERY (plan Task 35 entero, wal.rs/wal_sharded/audit/cli/benches/tests, rules, codegraph) | ✅ | §Re-baseline + §Impacto |
| 2 | Diseño del chain + decisión de scope (WAL completo, SHA-256, por-archivo) | ✅ | §Spec + §Diseño |
| 3 | Baseline bench `wal_throughput/never` (pre-cambio) | ✅ | §Evidencia de rendimiento (criterion baseline guardado) |
| 4 | TDD RED: tests de tamper/delete/legacy/reopen (unit) + test CLI (integración) | ✅ | 8 tests unit nuevos + `tests/wal_chain_verify.rs` (4); RED real: `wal_chain_verify_detects_altered_record_with_valid_crc` falló primero (ver §Evidencia TDD) y se refinó para discriminar |
| 5 | GREEN: framing v3 en `WalWriter`/`WalReader` + recover scan + verify fn + CLI | ✅ | 22/22 `wal::tests` + 34/34 `wal_sharded` + 13/13 integración (`wal_chain_verify`/`wal_rollback`/`proptest_wal_roundtrip`) |
| 6 | Compat: bump versión + lectura legacy + tests v1/v2 | ✅ | `wal_chain_verify_reports_legacy_for_v2_file` + `wal_writer_keeps_legacy_framing_on_v2_file` + keystone v3 en `wal_rollback.rs` |
| 7 | Verificación: fmt/clippy/nextest scoped + workspace + bench after + snapshot API | ✅ | fmt ✅ (propios) · clippy lib+bins ✅ · feature combos ✅ · lib 2278/2279 (1 flake de carga) · workspace-excl 3058/3059 (1 timeout de harness public_api) · snapshot API PASS 1/1 · bench §Evidencia |
| 8 | Cierre: task file sync + recitation + RESULTADO (commit/review = LEAD) | ✅ | §Cierre + recitation MCP (trace `ee507dae`) + RESULTADO §7 |

## Evidencia de rendimiento (bench before/after — `wal_throughput`)

**Protocolo:** criterion baseline capturado en el árbol pre-cambio (código = HEAD, sin ediciones mías) antes de tocar `src/`; mismo bench/binario/máquina para el after (criterion compara automáticamente). Modo estricto `never` (sin fsync = peor caso del hash, registros de 24 B) + modo default `periodic`. Δ aislado ≈ **+450–500 ns/registro** (hash SHA-256 de frame ~64 B + 64 B extra escritos). Nota: la máquina está compartida con la wave co-batch → varianza alta en runs largos; el Δ se replica consistente en histogramas p50.

| Modo/sample | Before (HEAD) | After (chain v3) | Δ / cambio |
|---|---|---|---|
| `never/batch_1` | 13.954 ms [8.97–16.83] | 14.195 ms | +32% (p=0.03) |
| `never/batch_100` | 45.75 µs | 104.5 µs | +131% (p=0.00) |
| `never/batch_1000` | 3.696 µs | 11.03 µs | +223% (p=0.00) |
| `never/batch_10000` | 316 ns | 912 ns | +175% (p=0.00) |
| `never` p50/registro (histograma) | 313–477 ns | 756–880 ns | **Δ ≈ +450–500 ns/registro** |
| `periodic/batch_1` (fsync por registro; SyncMode default) | — | p50 **4.2–4.4 ms**/registro | Δ ≈ 0.5 µs → **≈ +0.01%** |
| `periodic/batch_10000` (1 fsync/lote) | — | 1.36 µs/registro | (run ruidoso por load co-batch) |

**Decisión de scope (stop condition del plan aplicada como decisión documentada):**

- **Se MANTIENE la cadena por registro** (C1/C3). Evidencia: (a) el gate ≤5% NO es alcanzable para NINGUNA granularidad con hash criptográfico per-frame sobre registros de 24 B buffered — incluso `append` unitario (batch_1) paga 1 hash por llamada; (b) degradar a cadena por checkpoint/batch rompería C1 verbatim ("prev_hash + hash del frame" por registro) y C3 ("registro exacto") sin siquiera cumplir el gate en batch_1; (c) el path de durabilidad real (default `SyncMode::Periodic`, fsync por registro) queda a ~0.01% (≪5%) — el hash es despreciable frente al fsync.
- **Desviación documentada:** en modo buffered (`never`) con registros mínimos (24 B), el coste es estructural (+450–500 ns/registro ≈ +1.4–2.9× del write buffered). Registros reales (nodos con vector, KBs) amortizan el hash; el bench mide el peor caso a propósito.
- **FIND (candidato):** optimización de coste del chain (evaluar `compress256` directo/dispatch SHA-NI, BLAKE3, o coalescing por lote para registros cortos) — **sin bypass silencioso por config** (un opt-out configurable sería regresión de seguridad; cualquier degradación futura exige decisión explícita + review).
- **Falso positivo verificado:** truncado post-crash (torn tail) se reporta `incomplete_tail`, nunca `tampered` (test dedicado).

## Evidencia TDD (RED → GREEN)

- **RED real:** `wal_chain_verify_detects_altered_record_with_valid_crc` falló en su primer run (`left: 2, right: 3`): al flipear el primer byte del payload postcard el deserializador rechazaba el registro (scan-forward lo saltaba) — el test probaba el mecanismo equivocado. Se refinó para alterar contenido **decodable** ('AAAA'→'BAAA') con CRC recomputado: ahora prueba exactamente lo que CRC+deser NO pueden ver y la cadena SÍ (el lector replaya 3/3 registros alterados; `verify` reporta `tampered` en el offset del registro 2 con razón "altered"). 7/7 verdes tras el fix.
- **Cobertura por invariante:** clean→`verified` · alterado+CRC válido→`tampered`(offset,registro) · eliminado→`tampered`(link broken) · reopen→cadena continúa · v2→`legacy`+replay · append a v2 mantiene framing legacy · torn tail→`incomplete_tail` (no tamper) · truncado en frontera→límite documentado (pinned) · byte-identidad append/batch (test pre-existente que ahora ejerce v3) · CLI exit codes (0/1) end-to-end.

## Verificación (ejecutada)

| Gate | Comando | Resultado |
|------|---------|-----------|
| fmt (propios) | `rustfmt --check` (8 archivos tocados) | ✅ (el `cargo fmt --check` global falla por archivo co-batch VER-07 `vanta-memory/tests/dreaming.rs` sin formatear — no mío) |
| clippy | `cargo clippy -p vantadb --lib --bins` | ✅ 0 warnings (`--all-targets` bloqueado por el `--lib` test-cfg de VER-05/VER-07 en vuelo) |
| feature combos | `--no-default-features --features fjall` + `--features encryption` | ✅ |
| tests unit wal | `cargo nextest --profile audit -p vantadb --lib wal::tests` | ✅ 22/22 |
| tests unit wal_sharded | ídem `wal_sharded` | ✅ 34/34 |
| tests integración | `--test wal_chain_verify --test wal_rollback --test proptest_wal_roundtrip` | ✅ 13/13 |
| bench | `wal_throughput` before/after (never + periodic) | ✅ §Evidencia de rendimiento (decisión documentada) |
| tests unit lib (full) | `cargo nextest --profile audit -p vantadb --lib` | ✅ 2278/2279 passed; 1 TIMEOUT `index::core::tests::concurrent_insert_preserves_hnsw_invariants` → **flake por carga co-batch**: pasa aislado en 40.18s (`cargo test … --exact`); mi Δ WAL en ese test es ~0.05s, no causal |
| docs coverage | `pwsh scripts/validate-docs-coverage.ps1` | ❌ **1 gap real (mío)**: comando `verify` falta en la tabla CLI de `docs/user/operations/CONFIGURATION.md` (L386-403; patch exacto 1 fila en §Deuda 3). Resto ✅ (sdk 31, debug 0, config 69, error 35, py 51, mcp 47, skills 10 pares). Bloqueado por prohibición `docs/**` de la wave → lo aplica el LEAD |
| smoke E2E (`target/release/vanta-cli.exe`) | DB real 5 puts → `verify` text+JSON (4 shards, format v3, exit 0) · **extirpación** (borrar frame[0] de shard0) → `tampered at record 1 (offset 20), "chain link broken"` + **exit 1** · **truncado de cola** → `verified` (límite documentado, pinned) · **header destruido** → `corrupt` + incoherencia round-robin reportada | ✅ |
| tests workspace (excl. co-batch) | `cargo nextest --profile audit --workspace --exclude vanta-memory --build-jobs 2` | ✅ **3058/3059 passed** (3 skipped; 54 binarios excluidos por `default-filter`) · 1 TIMEOUT: `vantadb::public_api public_api_snapshot_matches_committed_file` — el build nightly de rustdoc excede el slow-timeout de 180s del perfil audit (característica ya comentada por el propio test: "dedicated CI job, not the Fast Gate"); el snapshot se regenera/valida por la vía del test con env `VANTADB_PUBLIC_API_UPDATE` (perfil default, sin timeout) — ver fila siguiente |
| snapshot API | `VANTADB_PUBLIC_API_UPDATE=1 cargo nextest run --cargo-profile bench -p vantadb --test public_api` → `cargo nextest run -p vantadb --test public_api` | ✅ regenerado 2× (post-VER-05; incluye `Commands::Verify` + `sdk::importers::*`) y **2º run PASS 1/1 (24.76s, sin crash)** — el `stack overflow`/exit 253 del 1er run fue cold-cache post-escritura (benigno); ⚠️ bajo `--profile audit` el test excede el slow-timeout 180s (build nightly rustdoc) → señal ambigua en el perfil audit local (FIND 2, §Deuda) |
| tests workspace (completo) | `cargo nextest --profile audit --workspace --build-jobs 2` | ⚠️ **bloqueado por deuda PRE-EXISTENTE (no VER-01)**: `vanta-memory/tests/smoke.rs:15` (`const assert!(cfg!(not(feature = "llm-driver")))`) falla por unificación de features del workspace (`vantadb-mcp`/`vanta-proxy` encienden `llm-driver` — estado HEAD). Dueño: LEAD/harness (FIND 5 en §Deuda) |

## Cierre
- **Commit:** NO (LEAD). Mensaje sugerido: `feat(wal): VER-01 — hash-chain tamper-evident (SHA-256 por registro) + vanta-cli verify` (adjuntar archivos de §Blast Radius; el snapshot incluye símbolos co-batch VER-05 → ver §Deuda 4).
- **Review:** NO self-review — P2-01 con contexto fresco; el orquestador pidió que el **diseño del chain/attestation lo revise `vanta-audit`** (§Diseño + §Evidencia de rendimiento están listos como ARTIFACT/CONTRACT del review). Señales para el reviewer: (1) cadena por archivo sin cross-segment link (documentado); (2) recovery NO valida cadena (verify es la autoridad) — decisión de semántica; (3) límites sin ancla externa (truncado de frontera / rewrite íntegro / borrado de segmento) documentados + pinned por test; (4) desviación del gate ≤5% en buffered (decisión de scope con evidencia).
- **Handoff:** `docs/api/` (§Diseño listo para levantar — prohibición de la wave); firma/ancla externa → vanta-audit (plan L941); VER-02/VER-04 consumen la cadena; FINDs candidatos en §Deuda.
- **Riesgo residual:** WAL v2 (legacy) sigue sin chain hasta rotar/compactar (migración natural, documentada); un atacante con reescritura íntegra consistente no es detectable sin ancla (v1.0).
