---
title: VantaDB Disaster Recovery Runbook
type: operations
status: active
tags: [vantadb, operations, dr]
last_reviewed: 2026-07-10
aliases: []
---

# VantaDB Disaster Recovery Runbook

## Overview

This runbook covers incident response and recovery procedures for VantaDB production deployments. It assumes a **single-node embedded deployment** (the primary deployment model). Multi-node replication is not yet supported.

---

## 1. Incident Severity Levels

| Level | Label | Impact | Response Time |
|-------|-------|--------|---------------|
| **SEV-1** | Critical | Data loss, complete outage | Immediate |
| **SEV-2** | High | Partial functionality lost | 1 hour |
| **SEV-3** | Medium | Degraded performance, minor data inconsistency | 4 hours |
| **SEV-4** | Low | Non-urgent anomaly | Next business day |

---

## 2. Recovery Procedures

### 2.1 SEV-1: Data Directory Missing or Corrupted

**Symptoms:**
- Engine fails to open with `IO error` or `corrupted data`
- WAL replay fails
- Startup crashes during index rebuild

**Diagnosis:**
```bash
# Check data directory integrity
ls -la /var/lib/vantadb/data/
vanta-cli doctor -d /var/lib/vantadb/data/
vanta-cli status -d /var/lib/vantadb/data/

# Check system logs
journalctl -u vantadb --since "1 hour ago" --no-pager
```

**Recovery:**

1. **Stop the service immediately** to prevent further writes:
   ```bash
   sudo systemctl stop vantadb
   ```

2. **Create a forensic copy** of the damaged directory:
   ```bash
   cp -a /var/lib/vantadb/data /var/lib/vantadb/data.corrupted-$(date +%F-%H%M)
   ```

3. **Restore from latest backup:**
   ```bash
   vanta-cli restore --from /backups/vantadb-latest --rebuild
   ```

4. **Verify restored data:**
   ```bash
   vanta-cli doctor -d /var/lib/vantadb/data/
   vanta-cli count -d /var/lib/vantadb/data/
   ```

5. **Restart the service:**
   ```bash
   sudo systemctl start vantadb
   ```

**If no backup exists (total loss):**
1. Document the extent of data loss
2. Determine the last known good state from application logs
3. Rebuild the database from source data (re-ingest documents, re-index)
4. Implement backup policy immediately after recovery (see [BACKUP_POLICY.md](BACKUP_POLICY.md))

---

### 2.2 SEV-1: Engine Crash Loop

**Symptoms:**
- Service repeatedly crashes on startup
- `systemctl status` shows `failed` state
- Journal shows process exit or panic

**Diagnosis:**
```bash
# Check recent logs
journalctl -u vantadb --since "1 hour ago" --no-pager | tail -100

# Attempt manual startup to see error
sudo -u vantadb vanta-cli server --http -d /var/lib/vantadb/data/
```

**Common causes:**

| Log Pattern | Likely Cause | Fix |
|-------------|-------------|-----|
| `Cannot acquire lock` | Another process holds the data dir lock | Kill stale process, or wait 1s for timeout |
| `Permission denied` | Wrong user/group on data files | `chown -R vantadb:vantadb /var/lib/vantadb/data` |
| `Out of memory` / `Killed` | Memory limit exceeded | Reduce `MemoryMax=` in systemd or container limits |
| `Address already in use` | Port conflict | Change `--port` or kill existing process |
| `Corrupted WAL segment` | WAL file corruption | See §2.3 |

**Recovery:**
1. Identify and fix the root cause from the table above
2. If the engine starts but crashes during index rebuild, try:
   ```bash
   # Rebuild indexes before starting the server
   vanta-cli rebuild-index -d /var/lib/vantadb/data/
   ```
3. If WAL corruption blocks startup, rename WAL files and let the engine start without replay (data since last flush is lost):
   ```bash
   mv /var/lib/vantadb/data/*.wal /tmp/wal-corrupted/
   ```
4. Verify with `doctor` and restart:
   ```bash
   sudo systemctl start vantadb
   ```

---

### 2.3 SEV-2: WAL Corruption

**Symptoms:**
- Engine logs `WAL corruption detected` during startup
- `doctor` reports inconsistent WAL state
- Some recent writes may be missing

**Recovery:**
```bash
# Stop service
sudo systemctl stop vantadb

# Run WAL repair
vanta-cli doctor -d /var/lib/vantadb/data/ --fix

# If doctor cannot fix:
# 1. Remove corrupted WAL segments (data since last flush is lost)
mv /var/lib/vantadb/data/*.wal /tmp/wal-corrupted/

# 2. Verify data integrity
vanta-cli doctor -d /var/lib/vantadb/data/

# 3. Restart
sudo systemctl start vantadb
```

**Post-recovery:**
- Compare current count with expected count
- Re-ingest any lost data from the source
- Check storage health (see §3)

---

### 2.4 SEV-2: Query Performance Degradation

**Symptoms:**
- Search latency > 2x baseline
- CPU usage at 100% for extended periods
- System logs show compaction backpressure

**Diagnosis:**
```bash
# Check database status
vanta-cli status -d /var/lib/vantadb/data/
vanta-cli stats -d /var/lib/vantadb/data/ --json

# Check system metrics
htop
iostat -x 1 5
```

**Recovery:**
1. **Rebuild HNSW index** (rebuilds BFS layout for cache locality):
   ```bash
   vanta-cli rebuild-index -d /var/lib/vantadb/data/
   ```
2. **Monitor compaction settings** — ensure the engine is not under compaction pressure:
   ```bash
   vanta-cli stats -d /var/lib/vantadb/data/ --json | grep compaction
   ```
3. **Adjust thread pool** to match available cores:
   ```bash
   export VANTADB_MAX_BLOCKING_THREADS=$(nproc)
   ```
4. **Check memory pressure** — if RSS is near `MemoryMax`, reduce `memory_limit`:
   ```bash
   export VANTADB_MEMORY_LIMIT=2147483648  # 2GB
   ```

---

### 2.5 SEV-3: Inconsistent Index

**Symptoms:**
- Text search returns fewer results than expected
- `doctor --deep` reports index inconsistencies
- `audit-index` finds mismatches between canonical records and the index

**Recovery:**
```bash
# Audit the text index
vanta-cli audit-index -d /var/lib/vantadb/data/ --deep

# Repair if inconsistencies are found
vanta-cli repair-text-index -d /var/lib/vantadb/data/

# Rebuild all indexes for thorough fix
vanta-cli rebuild-index -d /var/lib/vantadb/data/
```

---

### 2.6 SEV-3: Backup Failure

**Symptoms:**
- `vanta-cli backup` fails with an error
- Backup file is smaller than expected

**Diagnosis:**
```bash
# Check backup command output
vanta-cli backup --out /backups/vantadb-$(date +%F) 2>&1

# Verify existing backup integrity
vanta-cli restore --dry-run --from /backups/vantadb-latest
```

**Recovery:**
1. Ensure the backup target directory exists and is writable:
   ```bash
   ls -ld /backups/
   touch /backups/test-write && rm /backups/test-write
   ```
2. Check disk space:
   ```bash
   df -h /backups/
   ```
3. For the Fjall backend, backup requires a quiet moment — stop writes temporarily:
   ```bash
   sudo systemctl stop vantadb
   vanta-cli backup --out /backups/vantadb-$(date +%F)
   sudo systemctl start vantadb
   ```
4. Update backup automation (see [BACKUP_POLICY.md](BACKUP_POLICY.md))

---

## 3. Health Checks

### Scheduled Checks

| Frequency | Check | Command |
|-----------|-------|---------|
| Every 5 min | Service health | `curl -f http://localhost:8080/health` |
| Every 15 min | Database integrity | `vanta-cli doctor -d /data` |
| Every hour | Record count baseline | `vanta-cli count -d /data --namespace <ns>` |
| Every day | Deep index audit | `vanta-cli audit-index -d /data --deep` |
| Every day | Backup verification | `vanta-cli restore --dry-run --from /backups/latest` |

### Prometheus Alert Rules

```yaml
groups:
  - name: vantadb
    rules:
      - alert: VantaDBDown
        expr: up{job="vantadb"} == 0
        for: 1m
        labels: { severity: critical }

      - alert: VantaDBHighLatency
        expr: histogram_quantile(0.95, rate(vantadb_search_duration_seconds[5m])) > 0.5
        for: 5m
        labels: { severity: warning }

      - alert: VantaDBMemoryPressure
        expr: process_resident_memory_bytes{job="vantadb"} > 3.5e9
        for: 5m
        labels: { severity: warning }
```

---

## 4. Backup Strategy Summary

| Strategy | RPO | RTO | Method |
|----------|-----|-----|--------|
| Online CLI backup | 1 hour | 30 min | `vanta-cli backup --out ...` |
| Cold file copy | 0 (on stop) | 15 min | `rsync` of data directory |
| Automated schedule | 1 hour | 30 min | Cron + `backup` CLI |

See [BACKUP_POLICY.md](BACKUP_POLICY.md) for the full backup operational policy.

---

## 5. Post-Incident Checklist

After every incident:

- [ ] Document timeline (discovery, diagnosis, resolution)
- [ ] Identify root cause
- [ ] Verify backup integrity
- [ ] Update backup frequency if needed
- [ ] Add monitoring alerts if incident was not caught proactively
- [ ] If data was lost, document what and how much
- [ ] If procedure was missing from this runbook, add it
- [ ] Notify stakeholders (if applicable)
- [ ] Follow up with code fix or config change

---

## 6. Recovery Testing Schedule

| Test | Frequency | Procedure |
|------|-----------|-----------|
| Restore from backup | Monthly | Restore to a test instance, verify data |
| Failover simulation | Quarterly | Stop production, restore from backup, measure RTO |
| Index rebuild | Quarterly | Run `rebuild-index` and compare search results |
| WAL corruption recovery | Bi-annual | Inject WAL corruption in test environment, practice recovery |
| Full DR drill | Annual | Complete loss scenario: backup unavailable, rebuild from source data |

---

## 7. Contact Escalation

| Role | Contact Method | Responsibility |
|------|---------------|---------------|
| On-call engineer | PagerDuty / OpsGenie | Initial diagnosis, SEV-1/2 response |
| Database engineer | Slack / GitHub Issues | Data recovery, index repair |
| Engineering lead | Phone (if SEV-1) | Escalation, stakeholder communication |
| Security team | security@vantadb.dev | If data loss involves PII or credentials |

---

## 8. Appendices

### A. Quick Reference Card

```text
STOP ──► DIAGNOSE ──► RESTORE ──► VERIFY ──► RESTART
│         │            │            │
├─ systemctl stop     ├─ doctor    ├─ restore --rebuild  ├─ doctor
├─ forensic copy      ├─ stats     ├─ rebuild-index      ├─ count
└─ journalctl         ├─ audit     └─ re-ingest source   └─ search
```

### B. Common Commands

```bash
# Quick health
curl -f http://localhost:8080/health && vanta-cli doctor -d /data

# Full backup
vanta-cli backup --out /backups/vantadb-$(date +%F)

# Restore latest
vanta-cli restore --from /backups/vantadb-latest --rebuild

# Index repair
vanta-cli audit-index -d /data --deep
vanta-cli repair-text-index -d /data
vanta-cli rebuild-index -d /data

# Status
vanta-cli status -d /data
vanta-cli stats -d /data --json
```

### C. Recovery Script Template

```bash
#!/bin/bash
# DR recovery script — customize paths for your deployment
set -euo pipefail

DATA_DIR="${1:-/var/lib/vantadb/data}"
BACKUP_DIR="${2:-/backups}"
LOG="/var/log/vantadb-dr-$(date +%F-%H%M).log"

log() { echo "[$(date +%H:%M:%S)] $*" | tee -a "$LOG"; }

log "=== VantaDB DR Recovery ==="
log "Stopping service..."
sudo systemctl stop vantadb || true

log "Creating forensic copy..."
cp -a "$DATA_DIR" "${DATA_DIR}.corrupted-$(date +%F-%H%M)" || true

log "Finding latest backup..."
LATEST=$(ls -td "$BACKUP_DIR"/vantadb-* 2>/dev/null | head -1)
if [ -z "$LATEST" ]; then
    log "ERROR: No backup found at $BACKUP_DIR"
    exit 1
fi

log "Restoring from $LATEST..."
vanta-cli restore --from "$LATEST" --rebuild -d "$DATA_DIR"

log "Verifying..."
vanta-cli doctor -d "$DATA_DIR"
echo "ok" | vanta-cli count -d "$DATA_DIR"

log "Starting service..."
sudo systemctl start vantadb
sudo systemctl status vantadb --no-pager

log "=== Recovery complete ==="
```
