# VantaDB Cleanup Candidates - 2026-05-04

Policy: no definitive deletion was performed. These are candidates only.

| Priority | Path | Classification | Evidence | Recommended action |
|---:|---|---|---|---|
| P1 | `vantadb_data/` | Tracked local DB artifact | 17 tracked files, including `data/vector_store.vanta` at 64 MB. | Remove from version control only after confirmation; keep `/vantadb_data/` ignored. |
| P1 | `vanta_certification.json` | Generated certification evidence | 115.7 KB generated-looking root JSON. | Move under `docs/audits/` or `docs/operations/evidence/`, or quarantine if obsolete. |
| P2 | `test_sdk_db_*` directories | Ignored local test artifacts | 90 ignored directories found in repo root. | Quarantine or delete after confirmation; improve Python cleanup if new dirs continue accumulating. |
| P2 | `todo.md` | Ignored generated code snapshot | 1,028,576 bytes; produced by `dev-tools/collect_code.ps1`. | Keep ignored; consider moving output outside repo or under `.local/`. |
| P2 | `datasets/sift/` | Ignored local dataset | SIFT files include ~492 MB base and ~49 MB learn files. | Keep ignored; document download/rebuild flow only. |
| P2 | `target/` | Ignored build output | Contains multiple RocksDB artifacts near 964 MB each. | Keep ignored; no repo action. |
| P2 | `deep-research-report.md` | Strategic document | Overlaps with `docs/operations/*` and product-boundary docs. | Review for merge into operations docs or archive. |
| P2 | `seguimiento de proyecto.csv` | Project tracking document | Root-level planning CSV. | Move to `docs/operations/` or external tracker after review. |
| P3 | `test_runner.sh` | Container-specific script | Hard-codes `/app`, creates `/venv`, uses `apt-get`. | Keep only if Docker flow needs it; otherwise merge with Python wheel workflow docs. |
| P3 | `run_bench.sh` | Linux/root benchmark helper | Uses `apt-get`, overlaps with `cargo bench --no-run` and CI. | Move under `dev-tools/` or document as Linux-only. |
| P3 | `dev-tools/collect_code.ps1` | AI snapshot helper | Writes large ignored `todo.md`. | Keep if intentionally used; add warning in script/docs. |

## Quarantine Plan

If approved later, use `.local-quarantine/2026-05-04/` for local-only moves and add `.local-quarantine/` to `.gitignore`. Do not use `git rm`, `Remove-Item`, or permanent deletion without an explicit confirmation.

Suggested first confirmation batch:

1. Quarantine ignored `test_sdk_db_*` directories.
2. Quarantine or relocate root `todo.md`.
3. Decide whether `vantadb_data/` should be removed from Git tracking in a separate commit.
