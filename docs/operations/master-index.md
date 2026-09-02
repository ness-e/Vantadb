# Operations Master Index

**last_reviewed:** 2026-09-02

## docs/operations/

| File | Description |
|------|-------------|
| [AGENT_INSTRUCTIONS.md](AGENT_INSTRUCTIONS.md) | Instructions for AI agents working in this repo |
| [BACKUP_POLICY.md](BACKUP_POLICY.md) | Database and file backup procedures |
| [BACKUP_RESTORE.md](BACKUP_RESTORE.md) | End-user backup & restore guide (directory copy, JSONL export) |
| [BENCHMARKS.md](BENCHMARKS.md) | Performance benchmark results and methodology |
| [CI_POLICY.md](CI_POLICY.md) | Continuous integration policies |
| [COMMUNITY_GOVERNANCE.md](COMMUNITY_GOVERNANCE.md) | Community guidelines and governance model |
| [CONFIGURATION.md](CONFIGURATION.md) | System configuration reference |
| [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md) | Deployment procedures and checklist |
| [DISASTER_RECOVERY_RUNBOOK.md](DISASTER_RECOVERY_RUNBOOK.md) | Disaster recovery runbook |
| [DURABILITY_GUARANTEES.md](DURABILITY_GUARANTEES.md) | Data durability guarantees and SLAs |
| [EDITOR_INTEGRATIONS.md](EDITOR_INTEGRATIONS.md) | Editor/IDE integration setup |
| [EXPERIMENTAL_FEATURES.md](EXPERIMENTAL_FEATURES.md) | Experimental features documentation |
| [FUZZING.md](FUZZING.md) | Fuzzing setup and results |
| [GC_TTL.md](GC_TTL.md) | Garbage collection TTL configuration |
| [GRAFANA_SETUP.md](GRAFANA_SETUP.md) | Grafana dashboard setup |
| [grafana-dashboard.json](grafana-dashboard.json) | Grafana dashboard JSON definition |
| [MEMORY_TELEMETRY.md](MEMORY_TELEMETRY.md) | Memory telemetry and monitoring |
| [MCP_REGISTRY.md](MCP_REGISTRY.md) | MCP server.json manifest + registry submission state (MCP-40) |
| [PERFORMANCE_GUIDE.md](PERFORMANCE_GUIDE.md) | Performance optimization guide |
| [PERFORMANCE_TUNING.md](PERFORMANCE_TUNING.md) | Performance tuning parameters |
| [PILOT_PROGRAM.md](PILOT_PROGRAM.md) | Pilot program documentation |
| [PUBLIC_ISSUE_DRAFTS.md](PUBLIC_ISSUE_DRAFTS.md) | Public issue draft templates |
| [PYTHON_RELEASE_POLICY.md](PYTHON_RELEASE_POLICY.md) | Python SDK release policy |
| [RELIABILITY_GATE.md](RELIABILITY_GATE.md) | Reliability gate criteria |
| [REPO_CHECKLIST.md](REPO_CHECKLIST.md) | Repository maintenance checklist |
| [SECURITY.md](SECURITY.md) | Security policies and procedures |
| [SQLITE_MIGRATION_GUIDE.md](SQLITE_MIGRATION_GUIDE.md) | SQLite migration guide |
| [UPGRADE.md](UPGRADE.md) | Upgrade guide — version migration and backup before upgrade |
| [hardening.md](hardening.md) | Security hardening guide for VantaDB Server (production) |
| [master-index.md](master-index.md) | This index — canonical listing of all operations docs (self-indexed) |

## docs/archive/

| File | Description |
|------|-------------|
| [EXTRACCION-DOC-OLD-2026-08-05.md](../archive/EXTRACCION-DOC-OLD-2026-08-05.md) | Extracción de documentación legacy (2026-08-05) |
| [legacy-docs-investigacion-2026-07-16.md](../archive/legacy-docs-investigacion-2026-07-16.md) | Docs legacy de investigación (2026-07-16) |

> Optional: `docs/backlog-futuro.md` vive suelto en la raíz de `docs/` (no archivado); plan Open Core archivado en `docs/plans/archive/2026-08-06-oc-vantadb-pro.md`.

## Adiciones GOV-C5 (2026-08-22)

| Document | Description |
|----------|-------------|
| [chaos-testing.md](chaos-testing.md) | Chaos/failpoint testing guide (failpoint paths vigentes) |
| [ci-cd-guide.md](ci-cd-guide.md) | CI/CD setup and operations guide |
| [TEST_MAP.md](TEST_MAP.md) | Test map: qué suite correr por cambio (cifra canónica de tests) |
| [pilot-agreement-template.md](pilot-agreement-template.md) | Pilot program agreement template |
| [pilot-feedback-template.md](pilot-feedback-template.md) | Pilot feedback collection template |
| [pilot-onboarding-checklist.md](pilot-onboarding-checklist.md) | Pilot onboarding checklist |

> Regla (GOV-C5): todo doc nuevo en operations/ se agrega a este índice en el mismo PR. Este archivo se auto-indexa (master-index.md).
