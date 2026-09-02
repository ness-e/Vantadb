# Operations Master Index

**last_reviewed:** 2026-09-02
**files:** 35 `.md` + 1 `.json` (36 entries inc. self-indexed `master-index.md`) — taxonomía 6 categorías

> **Taxonomía operations (GOV-C5 26→35):** 26 (2026-08-22 baseline) → 32 (+6 GOV-C5: chaos-testing, ci-cd-guide, TEST_MAP, pilot×3) → 35 (+3 SRV-04/05: hardening.md + UPGRADE.md + self consistency, 2026-08-28). Este índice lista los 35 `.md` exactamente una vez.
> **Regla same-PR (GOV-C5):** todo doc nuevo en `operations/` se agrega a este índice en el **mismo PR**. Este archivo se auto-indexa (`master-index.md`).

## 1. Deploy & Config (6)

| File | Description |
|------|-------------|
| [AGENT_INSTRUCTIONS.md](AGENT_INSTRUCTIONS.md) | Instructions for AI agents working in this repo |
| [CONFIGURATION.md](CONFIGURATION.md) | System configuration reference |
| [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md) | Deployment procedures and checklist |
| [EDITOR_INTEGRATIONS.md](EDITOR_INTEGRATIONS.md) | Editor/IDE integration setup |
| [SQLITE_MIGRATION_GUIDE.md](SQLITE_MIGRATION_GUIDE.md) | SQLite migration guide |
| [UPGRADE.md](UPGRADE.md) | Upgrade guide — version migration and backup before upgrade |

## 2. Durability & Recovery (6)

| File | Description |
|------|-------------|
| [BACKUP_POLICY.md](BACKUP_POLICY.md) | Database and file backup procedures |
| [BACKUP_RESTORE.md](BACKUP_RESTORE.md) | End-user backup & restore guide (directory copy, JSONL export) |
| [DISASTER_RECOVERY_RUNBOOK.md](DISASTER_RECOVERY_RUNBOOK.md) | Disaster recovery runbook |
| [DURABILITY_GUARANTEES.md](DURABILITY_GUARANTEES.md) | Data durability guarantees and SLAs |
| [GC_TTL.md](GC_TTL.md) | Garbage collection TTL configuration |
| [RELIABILITY_GATE.md](RELIABILITY_GATE.md) | Reliability gate criteria |

## 3. Performance & Observability (5 + 1 json)

| File | Description |
|------|-------------|
| [BENCHMARKS.md](BENCHMARKS.md) | Performance benchmark results and methodology |
| [GRAFANA_SETUP.md](GRAFANA_SETUP.md) | Grafana dashboard setup |
| [grafana-dashboard.json](grafana-dashboard.json) | Grafana dashboard JSON definition |
| [MEMORY_TELEMETRY.md](MEMORY_TELEMETRY.md) | Memory telemetry and monitoring |
| [PERFORMANCE_GUIDE.md](PERFORMANCE_GUIDE.md) | Performance optimization guide |
| [PERFORMANCE_TUNING.md](PERFORMANCE_TUNING.md) | Performance tuning parameters |

## 4. Security & Governance (5)

| File | Description |
|------|-------------|
| [COMMUNITY_GOVERNANCE.md](COMMUNITY_GOVERNANCE.md) | Community guidelines and governance model |
| [EXPERIMENTAL_FEATURES.md](EXPERIMENTAL_FEATURES.md) | Experimental features documentation |
| [PUBLIC_ISSUE_DRAFTS.md](PUBLIC_ISSUE_DRAFTS.md) | Public issue draft templates |
| [SECURITY.md](SECURITY.md) | Security policies and procedures |
| [hardening.md](hardening.md) | Security hardening guide for VantaDB Server (production) |

## 5. CI / Testing & Quality (6)

| File | Description |
|------|-------------|
| [CI_POLICY.md](CI_POLICY.md) | Continuous integration policies |
| [FUZZING.md](FUZZING.md) | Fuzzing setup and results |
| [REPO_CHECKLIST.md](REPO_CHECKLIST.md) | Repository maintenance checklist |
| [TEST_MAP.md](TEST_MAP.md) | Test map: qué suite correr por cambio (cifra canónica de tests) |
| [chaos-testing.md](chaos-testing.md) | Chaos/failpoint testing guide (failpoint paths vigentes) |
| [ci-cd-guide.md](ci-cd-guide.md) | CI/CD setup and operations guide |

## 6. Programs & Registry (6)

| File | Description |
|------|-------------|
| [MCP_REGISTRY.md](MCP_REGISTRY.md) | MCP server.json manifest + registry submission state (MCP-40) |
| [PILOT_PROGRAM.md](PILOT_PROGRAM.md) | Pilot program documentation |
| [PYTHON_RELEASE_POLICY.md](PYTHON_RELEASE_POLICY.md) | Python SDK release policy |
| [pilot-agreement-template.md](pilot-agreement-template.md) | Pilot program agreement template |
| [pilot-feedback-template.md](pilot-feedback-template.md) | Pilot feedback collection template |
| [pilot-onboarding-checklist.md](pilot-onboarding-checklist.md) | Pilot onboarding checklist |

## Self-index

| File | Description |
|------|-------------|
| [master-index.md](master-index.md) | This index — canonical listing of all operations docs (self-indexed) |

## docs/archive/ (referenced, not counted in 35)

| File | Description |
|------|-------------|
| [EXTRACCION-DOC-OLD-2026-08-05.md](../archive/EXTRACCION-DOC-OLD-2026-08-05.md) | Extracción de documentación legacy (2026-08-05) |
| [legacy-docs-investigacion-2026-07-16.md](../archive/legacy-docs-investigacion-2026-07-16.md) | Docs legacy de investigación (2026-07-16) |

> Optional: `docs/backlog-futuro.md` vive suelto en la raíz de `docs/` (no archivado); plan Open Core archivado en `docs/plans/archive/2026-08-06-oc-vantadb-pro.md`.
