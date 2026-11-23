---
id: FIND-499
title: "migration.rs reconoce formatos actuales (plano y cifrado) de VantaFile, WAL e índice"
status: completed
type: fix
plan: docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md (Task 13)
---

# FIND-499 — Formatos actuales en integrity/migration checks

## Objetivo
`check_integrity` y la detección de migración deben aceptar las versiones
**actuales** (plana y cifrada) de cada artefacto y nunca marcar un artefacto
cifrado como migrable ni una versión futura como "corrupta".

## Causa raíz (verificada contra HEAD 1e3abcce9)
- `src/storage/vfile.rs:72` `VFILE_VERSION=2` · `:90` `VFILE_ENCRYPTED_VERSION=3`
- `src/wal.rs:26` `WAL_FORMAT_VERSION=3` · `:45` `WAL_ENCRYPTED_FORMAT_VERSION=4`
  · `:74` `WAL_POSTCARD_VERSION=1` (vive en `Header.schema_version`, NO en
  `format_version`) · `:60` `WAL_CHAIN_VERSION=3`
- `src/index/graph/types.rs:27` `VECTOR_INDEX_VERSION=8` · `:44` `VECTOR_INDEX_ENCRYPTED_VERSION=9`

Defectos en `src/migration.rs`:
(a) `check_integrity` compara `!=` contra UNA versión por artefacto: VantaFile
    vs 2, WAL vs `WAL_POSTCARD_VERSION` (=1, campo equivocado), índice vs 8.
    Una base cifrada (v3/v4/v9) y un WAL plano actual (v3) reportan falsas
    incidencias ("WAL at v3, latest is v1").
(b) `plan_all` usa `< VFILE_VERSION`, `!= VECTOR_INDEX_VERSION` (¡v9 se
    considera migrable! una reescritura con cabecera v8 lo corrompe) y
    `< WAL_POSTCARD_VERSION`.
(c) Rutas: `migrate_*` usan `>=`; se confirma que no reescriben cifrados.
    Además `migrate_wal` hacía bump de cabecera a `WalHeader::new(1)` — un
    bump v<3 → v3 es *intrínsecamente inseguro*: v3 cambia el framing
    (frames con hash-chain, `WAL_CHAIN_VERSION`). Se desactiva la reescritura
    por cabecera para el WAL (no-destructivo, documentado).

## Layout real del motor (verificado en `src/storage/engine/init.rs`)
- `data_dir = <db>/data` (init.rs:390).
- VantaFile: `data/vstore_L{0..3}.vanta` + legacy `data/vector_store.vanta` (VFLE).
- WAL: `data/vanta.wal` (1 shard) o `data/vanta.shardN.wal` (N>1); rotados
  `vanta.wal.<millis>` (excluidos deliberadamente: archivos históricos con
  range-based compat, no "formato actual").
- Índice: `data/vector_index.bin` (VNDX).
- Schema: `<db>/.vanta.schema` (raíz) — sin cambios.
- `vector_store.vanta`, `index.bin`, `wal.log` en raíz eran **código muerto**:
  el motor nunca escribió esos nombres en la raíz. Corregido a las rutas reales.

## Reglas de versión (citables por ADR-0060 / FIND-631)
Por artefacto: conjunto de versiones **actuales aceptadas** =
{plana_actual, cifrada_actual} (VantaFile {2,3}; WAL {3,4}; índice {8,9}).
- `v < plana_actual` → **NeedsMigration** (solo linaje plano; un artefacto
  cifrado nunca se migra implícitamente — ADR-0057 D7).
- `v ∈ {plana, cifrada}` → **Current**.
- `v > max(plana, cifrada)` → **Newer**: incidencia "formato más nuevo que
  este build", NUNCA "corrupto" (coherente con `found_version` de ADR-0057).
- Reglas centralizadas en `ArtifactSpec` + `classify()` para que
  `check_integrity` y `plan_all` no puedan divergir de nuevo.
- WAL: `WAL_POSTCARD_VERSION` NO es `format_version`; vive en
  `schema_version` del header y no participa en la clasificación.

## Impacto mapeado (Regla 0)
- Leídos completos: `src/migration.rs` (1100+ líneas), `src/wal.rs:1-229`,
  `src/storage/vfile.rs:50-109`, `src/index/graph/types.rs:15-59`,
  `src/storage/engine/init.rs` (init + `wal_paths`/`vfile_paths`/
  `detect_encryption_mode`), `src/wal_sharded.rs` (naming shards/rotados),
  `src/storage/engine/tests/encryption.rs` (helpers de DB cifrada).
- Referencias hacia dentro: constantes de versión (no se tocan), `Header`,
  `WalHeader`, `StorageEngine::open_with_config`, `Config::with_encryption`.
- Referencias entrantes: `cli_handlers/migrate.rs` (CLI `vanta-cli migrate`
  usa `plan_all`/`migrate_format`/`check_integrity`).
- Veredicto: cambio acotado a `src/migration.rs`. Blast radius = reportes de
  integrity/migration del CLI; sin cambio de formato on-disk.

## Steps
- [x] 1. Tests RED (a-d) + ajuste de tests obsoletos a rutas `data/` reales — RED: 8 fails
- [x] 2. Implementar `ArtifactSpec`/`classify` + paths reales en `plan_all`,
  `check_integrity`, `migrate_*`
- [x] 3. GREEN contrato (42/42) + clippy (-D warnings) + fmt — tras 2 nits de review
- [x] 4. Review P2-01 (vanta-review, fresh; (b) verificado: encrypted nunca migra)
- [x] 5. Commit local, sin push

## Contrato (verificación mecánica)
cargo nextest run --profile audit -p vantadb --features encryption -E "test(migration) or test(integrity)" --locked --build-jobs 2

## Tests nuevos
- `integrity_accepts_encrypted_v3_v4_v9` (cfg encryption): DB real cifrada → 0 incidencias.
- `integrity_accepts_plaintext_current_wal`: DB real sin clave → 0 incidencias (bug WAL vs v1).
- `migration_never_targets_encrypted_index`: header VNDX v9 → sin plan ni incidencia.
- `integrity_flags_future_version`: header v99 → incidencia "newer than this build".

## Riesgos
- Que `check_integrity` se use sobre `<db>` raíz y no `data/` → se chequea
  `data/` (layout real); los tests se escriben contra `data/`.
- Bump de cabecera WAL v<3→v3 corrompe framing → `migrate_wal` ya no reescribe
  cabeceras; avisa que la migración real requiere replay a segmento nuevo
  (no-destructivo; deuda documentada).

## SDP
SKILLS: systematic-debugging, test-driven-development (MCP campaign_* no
disponible — SDP manual por keywords del contrato: formatos on-disk,
versionado, integrity, migration)

## Review (P2-01)
- Tier: adversarial (`src/migration.rs` toca `src/wal*.rs` semantics / formatos
  on-disk — globs: `src/storage/**`, `src/wal*.rs`, `src/index/serialize/**`)
- Veredicto: **approve** (vanta-review, fresh context). Punto (b) verificado por
  construcción: `classify` da `NeedsMigration` solo a `v < plain_current`, y
  `encrypted_current > plain_current` en los tres specs → ningún artefacto
  cifrado entra en rutas de reescritura. Aplicados 2 nits: `wal_paths` filtra
  `is_file`, y `check_integrity` reporta `unreadable` en vez de tragar `Err`.
- Deuda conocida (aceptada): planes WAL son advisory — `migrate_wal` no
  reescribe (bump v<3→v3 corrompería el framing); sin feature `encryption`
  classify marca v3/v4/v9 como `Current` (el reader da el error explícito).

## Context Save Point
Si se interrumpe: releer "Reglas de versión" y "Layout real"; retomar desde el
primer step ⬜.
