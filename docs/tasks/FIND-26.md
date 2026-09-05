# FIND-26: PITR dead code (WalArchiver/PitrRestorer) — REMOVER

| Campo | Valor |
|---|---|
| Estado | ⏳ IN PROGRESS |
| Tipo | refactor (removal / dead code) |
| Contrato | wal_archiver.rs eliminado + 0 referencias colgantes + clippy/fmt limpios; decisión documentada |
| Decisión | **REMOVER** (lead, basada en RES-02 §2b: PITR necesita base snapshot + log replay, prerrequisito grande sin consumer; git history conserva el código) |

## Impacto mapeado (Regla 0)

**Archivos leídos completos:** `src/wal_archiver.rs` (496L), `docs/plans/2026-08-25-batch-backup-restore-chain.md` (130L).

**Grep exhaustivo** (`WalArchiver|PitrRestorer|wal_archiver|restore_to_timestamp|archive_segment|WalArchiveConfig` en TODO el workspace):

### Referencias de CÓDIGO (lo que rompe el compile):
| Ref | Naturaleza | Acción |
|---|---|---|
| `src/lib.rs:147-149` | export cfg-gated `pub mod wal_archiver` (`#[cfg(feature = "pitr")]`) | remover |
| `src/lib.rs:30` | fila `pitr` en tabla doc-comment de features | remover |
| `Cargo.toml:122-127` | feature `pitr = []` + comment gate (ADR-014) | remover |
| `src/wal_archiver.rs` | módulo completo (WalArchiver :56, PitrRestorer :219, tests inline :319-496) | borrar archivo |

**Veredicto STOP CONDITION:** NO disparado — cero call sites funcionales desde engine/SDK/CLI/bindings/tests externos (verificado con `rg` sobre `src/ tests/ benches/ vantadb-python/ vantadb-wasm/ vantadb-server/ vantadb-mcp/ vantadb-node/ vantadb-ts/`). RES-02 confirmó lo mismo. Dependencias usadas por el módulo (`web_time`, `serde`, `tempfile`) se usan masivamente en otros módulos → no queda nada huérfano.

### Referencias de DOCS vivos (marcar como removed/deferred):
| Doc | Línea | Acción |
|---|---|---|
| `docs/architecture/FEATURES.md:59` | fila `pitr` ⚠️ experimental | marcar ❌ removed |
| `docs/operations/EXPERIMENTAL_FEATURES.md:58` | PITR standalone API | marcar removed |
| `docs/architecture/adr/ADR-014-pitr.md` | ADR completo | nota de estado al inicio (superseded, git history) |
| `docs/Backlog.md:223` (FIND-26) y `:422` (CORE-02) | filas backlog | FIND-26 → resuelta remove; CORE-02 → nota de que requiere restore desde history |
| `.opencode/rules/durability.md:3` | scope list incluye `wal_archiver.rs (pitr)` | quitar de la lista |
| `.opencode/rules/concurrency-async.md:41` | menciona `wal_archiver.rs` en regla std::fs | quitar de la lista |
| `docs/strategy/VANTADB-PRO-FEATURES.md:19` | pitr candidata Pro `src/lib.rs:142` | marcar removed |
| `docs/operations/security/UNSAFE_INVENTORY.md:165` | inventario unwraps de wal_archiver.rs | quitar entrada |

### Referencias HISTÓRICAS (NO tocar — registros fechados):
`docs/reviews/*`, `docs/research/*`, `docs/plans/archive/*`, `docs/avance/historial/*`, `.opencode/skills/campaign-executor/tasks/{AUDREP-08,MCP-34,MCP-34b,FIND-25,MOD-06,RES-02,complete/*}.md`, memoria lessons/decisions, CHANGELOG. Son snapshots históricos, no docs vivos.

## Steps

1. ✅ Borrar `src/wal_archiver.rs` + limpiar `src/lib.rs` (:30, :147-149) + limpiar `Cargo.toml` (feature pitr)
2. ✅ Actualizar docs vivos (FEATURES.md, EXPERIMENTAL_FEATURES.md, ADR-014 → superseded, PRO-FEATURES, UNSAFE_INVENTORY, rules durability+concurrency-async, Backlog FIND-26 resuelta + CORE-02 bloqueada-con-nota)
3. ✅ Verify full: fmt --check EXIT 0 · clippy --workspace --all-targets --all-features -D warnings EXIT 0 (2m51s) · nextest wal/storage/snapshot 432/432 PASS (77.7s) · grep 0 refs en src/tests/benches/bindings/Cargo.toml (exit 1) · cargo check EXIT 0
4. ⬜ Cierre: plan file estado + recitation + memoria (commit lo hace el lead)

## Context Save Point

Tarea COMPLETA pendiente de commit del lead. Archivos tocados:
- Borrado: `src/wal_archiver.rs`
- Código: `src/lib.rs`, `Cargo.toml`
- Docs: `docs/architecture/FEATURES.md`, `docs/operations/EXPERIMENTAL_FEATURES.md`, `docs/architecture/adr/ADR-014-pitr.md` (status superseded), `docs/strategy/VANTADB-PRO-FEATURES.md`, `docs/operations/security/UNSAFE_INVENTORY.md`, `.opencode/rules/durability.md`, `.opencode/rules/concurrency-async.md`, `docs/Backlog.md` (FIND-26 + CORE-02)

