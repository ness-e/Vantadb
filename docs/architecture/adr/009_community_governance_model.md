---
title: "ADR 009: Community Governance Model and Contribution Process"
type: adr
status: active
tags: [vantadb, architecture, adr]
last_reviewed: 2026-07-10
aliases: []
---

# ADR 009: Community Governance Model and Contribution Process

## Status

Approved.

## Context

As an open-source embedded vector database, VantaDB requires a formal governance structure. See [[../../operations/COMMUNITY_GOVERNANCE.md|Community Governance & SLA Policy]] for the current operational policy.

This ADR captures the decision rationale behind the governance model.

## Decision

1. **BDFL + Core Team model** with graduated responsibility (Committer → Core Team → BDFL).
2. **CLA requirement** for contributions >15 lines (Individual + Corporate CLA).
3. **RFC process** for major changes (7-day comment period, Core Team vote).
4. **Protected `main` branch** — requires CI, squash merge, Core Team review.
5. **Contributor Covenant v2.1** as Code of Conduct.

## Rationale

- **Clear decision path** avoids paralysis by consensus while requiring broad buy-in for major changes.
- **IP clarity** via CLA ensures clean licensing.
- **Quality gate via RFC** catches design issues before code is written.
- **Graduated responsibility** gives contributors a visible growth path.

## Consequences

### Benefits
- Well-defined escalation path and decision-making
- Clean licensing via CLA enforcement
- Wasted-implementation prevention via RFC process

### Costs
- BDFL bus factor (mitigated by documented ADRs 001-008)
- RFC overhead for small changes (48h expedited path for security patches)
- Maintainer burnout risk (shared rotation, capped review workload)
- CLA friction (15-line threshold reviewed quarterly)
