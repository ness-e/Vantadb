---
title: "VantaDB Community Governance & SLA Policy"
type: operations
status: active
tags: [vantadb, operations]
last_reviewed: 2026-07-01
aliases: []
---

# VantaDB Community Governance & SLA Policy

This document establishes the official governance rules, contribution workflows, and maintainer SLA commitments for **VantaDB** as an Open-Core system (Apache-2.0). Our goal is to ensure a transparent, active, and welcoming community for external developers and systems engineers.

---

## 1. Issue Triage and Labels Workflow

When a new issue is submitted to the VantaDB GitHub repository, it undergoes a structured triage process within our maintainer response window.

### Issue Triaging Flow:
```
New External Issue ──► [ Triage Label Added ] ──► Assigned to Milestone
                               │
            ┌──────────────────┴──────────────────┐
            ▼                                     ▼
     Confirmed Bug                         Feature Request
     - Label: 'bug'                        - Label: 'feature'
     - Target: Hotfix / Patch              - Target: RFC / Next Release
```

### Standard Labels Catalog:
* **`triage`:** Automatically applied to new incoming issues requiring categorization.
* **`bug`:** Confirmed defects, crashes, or incorrect behavior.
* **`feature`:** Proposed enhancements or additions to core systems.
* **`good first issue`:** Isolated tasks with low architectural context requirements, ideal for new external contributors.
* **`triage:waiting-for-user`:** Pending input, reproduction scripts, or logs from the author.
* **`packaging`:** Issues related to Maturin, Python wheels, compilation, or PyPI.

---

## 2. Maintainer Response SLA Commitments

To prevent community attrition and maintain high engagement, the core maintainers commit to the following Service Level Agreements (SLAs) for public interactions:

| Activity | SLA target | Description |
|---|---|---|
| **Issue Triage** | **< 48 Hours** | Every new issue will be acknowledged, categorized, and have a triage label added. |
| **Pull Request Review** | **< 48 Hours** | Submitted PRs will receive an initial technical review with actionable feedback or approval. |
| **Discord / Forum Help** | **< 48 Hours** | Technical questions in the official Discord support channels will receive maintainer assistance. |

---

## 3. External Contribution Lifecycle

We encourage developers to propose optimizations, fix bugs, and build SDK integrations. To maintain code quality and safety, all contributions must follow this lifecycle:

### Step 1: Fork and Branch
1. Fork the [ness-e/Vantadb](https://github.com/ness-e/Vantadb) repository.
2. Create a branch named after the issue or feature: `feat/issue-number-title` or `fix/issue-number-bugname`.

### Step 2: Quality Gates (Pre-commit checklist)
Before submitting a Pull Request, verify that all local checks pass:

```bash
# 1. Format code using the workspace style guidelines
cargo fmt --all

# 2. Run Clippy without warnings (warnings are treated as errors)
cargo clippy --all-targets --all-features -- -D warnings

# 3. Run all unit, integration, and certification tests
cargo test --workspace --release

# 4. Verify dependency licenses and audit vulnerabilities
cargo deny check
```

### Step 3: Pull Request Submission
* **Title:** Concise title describing the change (e.g., `feat(index): add SIMD support for Euclidean distance`).
* **PR Description:** Reference the issue it closes (`Closes #123`), explain the technical trade-offs, and provide evidence that local tests pass.
* **Review Process:** Within our 48-hour SLA, a maintainer will review the PR. Once approved and CI builds pass, it will be merged into `main`.

---

## 4. Moderation Policy

We follow a professional and technical Code of Conduct. Harassment, offensive language, or spam in GitHub issues, Pull Requests, or official Discord channels will not be tolerated. Maintainers reserve the right to lock issues, reject PRs, or ban accounts violating these standards.
