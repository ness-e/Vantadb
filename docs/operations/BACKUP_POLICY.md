# VantaDB Backup and Snapshot Policy

## Overview

VantaDB supports different storage backends, each with unique operational trade-offs regarding performance, latency, and data protection. As VantaDB modernizes its storage layer (e.g., migrating to Fjall), the operational contract for creating database backups changes.

This document outlines the official policy for safely backing up VantaDB data depending on the active backend.

## Policy by Backend

### 1. Fjall Backend (Current Default)

**Live Checkpoints / Snapshots**: ❌ **NOT SUPPORTED**

**Why?**
The Fjall engine prioritizes low-latency ingestion and lock-free concurrency. In its current version, it does not expose a native API for creating point-in-time consistent physical snapshots of the SSTables while the database is actively writing.

**How to Backup:**
Operators MUST NOT rely on `create_life_insurance` (native snapshots). Attempting to do so will safely abort with an explicit error.
Instead, use one of the following strategies:

1. **Volume-Level Snapshots (Recommended):** Use your cloud provider's snapshot capability on the underlying disk (e.g., AWS EBS Snapshots, GCP Persistent Disk Snapshots) or OS-level filesystem snapshots (like ZFS/LVM). Fjall is crash-safe; a volume snapshot will capture a crash-consistent state that VantaDB can recover from.
2. **Cold Backups:** Safely shut down the VantaDB process and create a standard copy/tarball of the data directory.

Cold-copy restore is now part of the fast validation suite for the default
Fjall path. The restore check reopens the copied directory and verifies
canonical memory records, BM25/phrase text search, and Hybrid Retrieval v1.

### 2. RocksDB Backend (Explicit Fallback)

**Live Checkpoints / Snapshots**: ✅ **SUPPORTED**

**Why?**
RocksDB provides a native `Checkpoint` API that hardlinks SSTables to create a point-in-time snapshot without stopping the database.

**How to Backup:**
Operators can continue using the `create_life_insurance` API. This will instantly create a consistent snapshot directory without downtime.

### 3. InMemory Backend (Testing Only)

**Live Checkpoints / Snapshots**: ❌ **NOT SUPPORTED**

Data is ephemeral and lost upon shutdown. No backup mechanism is provided or needed.

## Manual Compaction Policy

Like backups, **manual compaction** is explicitly governed by the backend capabilities:

- **RocksDB**: Supports manual compaction. The VantaDB maintenance worker will periodically trigger this to aggressively reclaim disk space from tombstones.
- **Fjall**: Does not support manual compaction triggers. Fjall's internal LSM tree aggressively self-manages compaction in the background. The maintenance worker will log an expected degradation (an informational skip) rather than attempting to force a compaction.

## Compatibility and Migration

As of the current version, **Fjall is the default backend**.

**New Installations:**
All new instances created via `StorageEngine::open()` or by using `EngineConfig::default()` will automatically initialize with Fjall.

**Existing Deployments:**
If a directory is already populated with RocksDB data, attempting to open it with Fjall as the default will result in an initialization error. To safely upgrade or maintain an existing database:
- **To keep using RocksDB:** You MUST explicitly configure `BackendKind::RocksDb` in your `EngineConfig` before opening the engine.
- **To migrate to Fjall:** A logical migration (exporting data and re-importing into a new Fjall-backed instance) is required.

## JSONL Export Is Not a Physical Backup

`export_namespace`, `export_all`, and `vanta-cli export` serialize canonical
memory records only. They intentionally do not serialize HNSW files, backend
SSTables, WAL files, derived namespace indexes, or text-index internals.

Use JSONL for logical movement, portability, and rebuildable imports. Use
backend snapshots or cold copies when the requirement is physical backup and
restore of an embedded database directory.

## Operator Restore Check

For Fjall cold backups:

1. Stop the process or close the embedded handle.
2. Copy the full database directory to the restore location.
3. Open the restored directory with `VantaEmbedded::open`.
4. Run `vanta-cli audit-index --db <restored> --json`.
5. Run representative memory search checks for vector-only, text-only, and
   hybrid retrieval.

If the audit reports drift, run `vanta-cli rebuild-index --db <restored>` and
repeat the audit before treating the restore as healthy.
