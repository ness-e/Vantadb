---
title: "ADR 009: Community Governance Model and Contribution Process"
type: adr
status: active
tags: [vantadb, architecture, adr]
last_reviewed: 2026-07-03
aliases: []
---

# ADR 009: Community Governance Model and Contribution Process

## Status

Status: Approved

## Context

As an open-source embedded vector database, VantaDB requires a formal governance structure to manage contributions, maintain code quality, resolve disputes, and ensure long-term project sustainability. The governance model must address:

1. **Contribution Volume:** The core is Rust code with significant algorithmic complexity (HNSW graphs, LSM storage, WAL recovery). Unsolicited contributions require review bandwidth and quality gates.
2. **API Stability:** VantaDB exposes a public API surface across Rust, Python, and MCP protocol layers. Breaking changes must be carefully coordinated.
3. **Security Sensitivity:** The engine handles persistent data. Cryptographic integrity of WAL records, buffer overflow prevention, and correct concurrent access patterns are critical.
4. **Community Trust:** Contributors need clear visibility into decision-making, maintainer selection, and project direction to invest their time confidently.
5. **Intellectual Property:** Contributions must be accompanied by clear licensing grants to keep the project's Apache 2.0 / MIT dual-licensing clean.

## Decision

1. **Benevolent Dictator for Life (BDFL) with Core Team:** VantaDB adopts a lightweight BDFL model with an expanding Core Team:
   - **BDFL:** Holds final authority on design decisions, API changes, and conflict resolution. Intended to transition to a steering committee as the community grows.
   - **Core Team (3-5 members):** Granted merge permissions on `vantadb-server` and `vantadb-core`. Core team members are nominated by the BDFL after sustained contribution history (>6 months) and confirmed through a simple majority of existing Core Team members.
   - **Committers (unlimited):** Granted merge permissions on ancillary crates (`vantadb-python`, `vantadb-cli`, documentation). Committer status is granted by any Core Team member after demonstrated competence.

2. **CLA Requirement:** All contributors whose changes exceed 15 lines of code must sign an Individual Contributor License Agreement (CLA) managed via GitHub CLA assistant. The CLA grants:
   - A non-exclusive, royalty-free license to the project to use, modify, and distribute the contribution.
   - A patent peace provision: the contributor agrees not to assert patents they control against downstream users of the contributed code.
   Corporate contributions require a Corporate CLA signed by an authorized representative of the entity.

3. **RFC Process for Major Changes:** The following changes require a formal RFC (Request for Comments) before implementation:
   - Any change to the public API surface (Rust `pub fn`, Python SDK public methods, MCP protocol messages).
   - Storage format or WAL record layout modifications that break backward compatibility.
   - New storage backends or database drivers.
   - Dependency additions that change the licensing posture (e.g., adding a GPL-licensed crate).
   
   The RFC process is:
   1. Contributor opens a PR against `docs/rfc/` with a structured RFC document.
   2. RFC is discussed for a minimum 7-day comment period.
   3. Core Team votes (simple majority). Tie broken by BDFL.
   4. Accepted RFCs are merged and tracked as "RFC-Accepted" in the project board.

4. **Repository Structure and Access Control:**
   - `main` branch is protected: requires passing CI (all lints, tests on both backends), a squash merge policy, and at least one Core Team review.
   - `develop` branch (if maintained) is protected with the same rules except review requirements may be reduced to one Committer review.
   - Release branches (`release/v*`) are created by the Core Team and accept only bug-fix cherry-picks.

5. **Code of Conduct:** The project adopts the [Contributor Covenant v2.1](https://www.contributor-covenant.org/) as its Code of Conduct. Violations are reported to the Core Team via a private email alias and are resolved confidentially.

## Consequences

### Benefits

- **Clear Decision Path:** The BDFL + Core Team structure provides a well-defined escalation path that avoids paralysis by consensus while still requiring broad buy-in for major decisions.
- **IP Clarity:** CLA enforcement ensures the project maintains clean licensing and can relicense if necessary (e.g., to adopt a more permissive future license).
- **Quality Gate via RFC:** Complex changes are debated before code is written, reducing wasted implementation effort on designs that will not be accepted.
- **Graduated Responsibility:** The Committer -> Core Team -> BDFL ladder gives contributors a visible path to increased responsibility without requiring immediate deep architectural knowledge.

### Technical Debt / Costs

- **BDFL Bus Factor:** The project has a single point of philosophical authority. Documentation of design principles (see ADR 001-008) mitigates this by encoding past decisions in a reviewable format.
- **RFC Overhead:** Small but controversial changes may be slowed by the 7-day RFC minimum. The Core Team may grant an expedited 48-hour process for urgent security patches.
- **Maintainer Burnout Risk:** Without dedicated review rotation, review bottlenecks may form. The Core Team commits to a shared rotation schedule and explicitly caps review workload to prevent burnout.
- **CLA Friction:** The 15-line threshold aims to keep trivial contributions (typo fixes, minor doc improvements) unfettered while protecting against substantial unattributed code drops. The exact threshold will be reviewed quarterly based on contribution patterns.
