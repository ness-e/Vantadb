---
title: Backup & Restore Guide
type: operations
status: active
tags: [vantadb, operations, backup, restore]
last_reviewed: 2026-08-23
aliases: []
---

# Backup & Restore Guide

How to back up your embedded VantaDB data and restore it. This is the
end-user guide; for production incident response see
[DISASTER_RECOVERY_RUNBOOK.md](DISASTER_RECOVERY_RUNBOOK.md) and
[BACKUP_POLICY.md](BACKUP_POLICY.md).

## What to back up

Everything VantaDB persists lives in **one data directory** (the path you pass
when opening the database, e.g. `./vanta_data`). Backing up that directory is
backing up your database.

**When to back up:**

- **Before upgrading** — see [UPGRADE.md](UPGRADE.md) ("backup before upgrade").
- **Before destructive operations** — bulk deletes, namespace drops, WAL
  manipulation, index rebuilds.
- On a regular schedule if data is valuable (see [BACKUP_POLICY.md](BACKUP_POLICY.md)).

## Method 1 — Copy the data directory

Simplest method. Stop writes first so the copy is consistent:

1. Close the database / stop the process holding it (Python: `del db` or exit
   the process; server: stop the service).
2. Copy the directory:

   ```bash
   cp -r ./vanta_data ./vanta_data.backup
   ```

3. Restart your application.

Fast and byte-faithful, but tied to the VantaDB version that wrote it.

## Method 2 — Export to portable JSONL

Exports every record as JSONL — readable text, portable across versions:

```python
db = vantadb.VantaDB("./vanta_data")
report = db.export_all("backup.jsonl")
```

```typescript
const report = db.exportAll("./backup.jsonl");
```

Both return a report with `records_exported`. Restore later with `import_file()` /
`importFile()`. Prefer this before upgrades: the JSONL format is version-independent.

## Restoring

**From a directory copy:** point VantaDB at the backup directory (or swap it
back into place while stopped):

```python
db = vantadb.VantaDB("./vanta_data.backup")
```

**From a JSONL export:** open a fresh database and import:

```python
db = vantadb.VantaDB("./restored_data")
report = db.import_file("backup.jsonl")
print(report["inserted"], report["updated"])
```

## Verify your backup

A backup you haven't verified is a hope. After each backup, restore it to a
**temporary** directory and read from it:

1. Import/copy the backup into a temp location (`/tmp/verify`).
2. Open it and check a known record:

   ```python
   vdb = vantadb.VantaDB("/tmp/verify")
   print(vdb.get("my_namespace", "known-key"))  # must return your record
   ```

3. Compare record counts against the live database, then delete the temp dir.

This put → backup → restore → doctor → get procedure is validated end-to-end
(GOV-A3, verified 2026-08-22 — see [Daily Backup Verification in the DR
runbook](DISASTER_RECOVERY_RUNBOOK.md#daily-backup-verification)), which also
documents the `vanta-cli` equivalent (`backup`, `restore`, `doctor`, `count`,
`get`) for CLI-based deployments.

## Which method when

| Situation | Method |
|-----------|--------|
| Quick local safety copy | Method 1 (directory copy) |
| Before an upgrade | Method 2 (JSONL — survives version changes) |
| Scheduled automated backups | Either; JSONL for long-term retention |
| Moving data to another machine/version | Method 2 |
| Production incident response | [DR runbook](DISASTER_RECOVERY_RUNBOOK.md) procedures |
