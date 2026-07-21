---
title: "ADR 0001: Adoptamos ADRs — Architecture Decision Records"
type: adr
status: accepted
tags: [vantadb, architecture, adr, meta]
last_reviewed: 2026-07-21
aliases: [ADR-0001, meta-adr]
---

# ADR 0001: Adoptamos ADRs — Architecture Decision Records

## Status

Status: Accepted

## Context

VantaDB had grown to span multiple crates, SDKs, bindings, and adapters before any formal decision-recording process existed. As a result:

- **Undocumented decisions** accumulated silently. Choices about storage backends, HNSW parameters, WAL strategies, and binding architectures were made but never captured with their rationale.
- **The same debates repeated.** Without written context, team members (human and agent) revisited trade-offs that had already been resolved, wasting cycles re-deriving conclusions.
- **Newcomers lacked historical context.** Onboarding required verbal hand-offs and digging through commit messages to understand why the architecture was the way it was.
- **Agent context was brittle.** AI agents joining the project had no structured record of architectural intent, leading to proposals that contradicted settled decisions.

Nine ADRs already existed (ADR 001–009), but the format, location, template, and process were implicit — never codified. This ADR documents the decision to adopt ADRs formally, providing the canonical template, storage location, and creation workflow for all future ADRs.

## Decision

We adopt **Architecture Decision Records (ADRs)** as the formal mechanism for documenting significant architectural decisions in VantaDB.

### Format

Nygard-style ADRs (Michael Nygard's lightweight architectural decision record format), with the following structure:

1. **Title:** Sequential number and descriptive title (`ADR NNN: Title`)
2. **Status:** Proposed | Accepted | Superseded by ADR-NNN | Deprecated
3. **Context:** The forces at play, including technological, organizational, and project-specific constraints
4. **Decision:** The chosen approach, described in sufficient detail
5. **Consequences:** Resulting context, trade-offs, and follow-up work
6. **Alternatives Considered** (optional but encouraged): Options weighed and why they were rejected

### Location

All ADRs are stored in:

```
docs/architecture/adr/
```

Named with the pattern:

```
NNN_terse_kebab_case_description.md
```

For example:

```
005_hnsw_parameters.md
```

### Template

Every new ADR MUST follow this template:

```markdown
---
title: "ADR NNN: Title"
type: adr
status: <proposed|accepted|deprecated|superseded>
tags: [vantadb, architecture, adr]
last_reviewed: YYYY-MM-DD
aliases: []
---

# ADR NNN: Title

## Status

Status: <Proposed | Accepted | Superseded by ADR-MMM | Deprecated>

## Context

<!-- Describe the forces at play, relevant background, and why this decision needs to be made. Include concrete constraints and requirements. -->

## Decision

<!-- Describe the chosen approach in sufficient detail. Number multiple sub-decisions for clarity. -->

## Consequences

### Benefits

<!-- What becomes easier, what improves, what positive outcomes follow. -->

### Technical Debt / Costs

<!-- What trade-offs were accepted, what follow-up work is deferred, what regressions are known. -->

## Alternatives Considered

<!-- Optional but encouraged. List each alternative with its pros and trade-offs, and state why it was rejected. -->

### Alternative A

- **Pros:** ...
- **Cons:** ...
- **Rejected because:** ...

### Alternative B

- **Pros:** ...
- **Cons:** ...
- **Rejected because:** ...
```

## Consequences

### Benefits

- **Decisions are auditable.** Anyone (human or agent) can read the rationale behind any architectural choice, reducing tribal knowledge.
- **Debate is captured.** Trade-offs and rejected alternatives are recorded, preventing re-litigation of settled questions.
- **Onboarding accelerates.** New contributors can read the ADR series to understand the project's architectural evolution.
- **Agent alignment improves.** AI agents can load ADRs before proposing changes, avoiding conflicts with past decisions.
- **Commit messages can link to ADRs.** PR descriptions and commit messages reference ADR numbers for traceability.

### Process

Creating a new ADR follows this workflow:

1. **Branch:** Create a feature branch from `develop` (or relevant working branch).
2. **Draft:** Copy the template from ADR-0001 into `docs/architecture/adr/NNN_kebab_name.md`.
3. **Write:** Fill in Context, Decision, Consequences, and (optionally) Alternatives Considered.
4. **Review:** Open a PR. The ADR is reviewed for clarity, completeness, and consistency.
5. **Accept:** On approval, the ADR's status changes from `proposed` to `accepted` and is merged.

### Technical Debt / Costs

- **Process overhead.** Writing an ADR takes 15–60 minutes per decision. This is a deliberate investment: the cost of not documenting is paid repeatedly in re-debate and misalignment.
- **Maintenance burden.** ADRs must be reviewed periodically; `last_reviewed` dates track staleness. Stale ADRs should be revisited or explicitly deprecated.
- **Discipline required.** The team must enforce the rule: "no significant architectural decision without an ADR." Enforcement relies on code review and the pre-launch certification gate (`vantadb-certify` layer 6: docs review).

## Alternatives Considered

### No formal process (status quo)

- **Pros:** Zero overhead, no new process to learn.
- **Cons:** Decisions remain tacit, debates repeat, onboarding remains slow. This was already causing measurable friction.
- **Rejected because:** The cost of re-litigation exceeded the cost of documentation.

### Wiki / Notion-based documentation

- **Pros:** Rich formatting, collaborative editing.
- **Cons:** Not in version control, no PR workflow, no direct link to code. Tends to drift from reality.
- **Rejected because:** ADRs live with the code, are reviewed like code, and remain in sync with the codebase.

### Google Docs

- **Pros:** Easy to share, rich review tools.
- **Cons:** Not in version control, no structured format, discoverability degrades over time.
- **Rejected because:** Same weaknesses as wiki, plus access-control fragmentation.
