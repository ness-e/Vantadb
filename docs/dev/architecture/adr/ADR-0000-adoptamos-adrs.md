---
title: "ADR-0000: Adoptamos ADRs — Architecture Decision Records"
kind: adr
status: accepted
description: "Status: Accepted"
aliases: [ADR-0000, ADR-0001, meta-adr]
tags: [vantadb, architecture, adr, meta]
---

# ADR-0000: Adoptamos ADRs — Architecture Decision Records

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

1. **Title:** Sequential number and descriptive title (`ADR-NNNN: Title`)
2. **Status:** Proposed | Accepted | Rejected | Superseded by ADR-NNNN | Deprecated
3. **Context:** The forces at play, including technological, organizational, and project-specific constraints
4. **Decision:** The chosen approach, described in sufficient detail
5. **Consequences:** Resulting context, trade-offs, and follow-up work
6. **Alternatives Considered** (optional but encouraged): Options weighed and why they were rejected

### Location

All ADRs are stored in:

```
docs/dev/architecture/adr/
```

Named with the pattern:

```
ADR-NNNN-terse-kebab-case-description.md
```

Four digits, zero-padded. Title-case words joined by single hyphens, lowercase.
The number is unique across the whole directory — including ADRs that arrived with a
`COMP-`/`DRV-` prefix from a campaign, which are renumbered into the same space on
adoption.

For example:

```
ADR-0005-hnsw-parameters.md
```

**Amendment (2026-09-29).** This section originally prescribed
`NNN_terse_kebab_case_description.md` (e.g. `005_hnsw_parameters.md`). That
prescription is withdrawn: it is the reason the corpus was split across three naming
styles, and the split is now closed.

- `0000` is reserved for this meta-ADR (the one describing the ADR process itself).
- Every ADR, whatever series it arrived from, takes a number from the same space.
  A number is never reused or shared by two decisions; the 041 collision
  (`041_anti_stutter.md` vs `ADR-0041-error-variant-renames.md`, backlog `HIG-02`)
  resolved in favour of the nine inbound references: the anti-stutter umbrella
  became `ADR-0047`.
- Slugs are lowercase kebab-case. Version numbers and dotted identifiers inside a
  slug keep their dots (see `ADR-0044-acumulado-breaking-0.6.0-find-133.md`).

The amendment landed in the same pass as the rename, per the owner's approval. Every
file in the directory now follows the pattern above.

### Template

Every new ADR MUST follow this template:

Updated 2026-09-29 for `kind`/`status` vocabulary, the filename pattern and the
three-sources-of-truth rule. The template keys are now the ones
`docs/_schema/frontmatter.schema.json` validates; `type` and `last_reviewed` are
retired (see that schema's `$comment`).

```markdown
---
title: "ADR-NNNN: Title"
kind: adr
status: <proposed|accepted|rejected|deprecated|superseded>
description: One line, no trailing period, <=250 chars.
tags: [vantadb, architecture, adr]
aliases: []
supersedes: docs/dev/architecture/adr/ADR-NNNN-slug.md   # optional
---

# ADR-NNNN: Title

## Status

Status: <Proposed | Accepted | Rejected | Superseded by ADR-NNNN | Deprecated>

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
2. **Draft:** Copy the template from ADR-0000 into `docs/dev/architecture/adr/ADR-NNNN-kebab-name.md`, using the next free four-digit number.
3. **Write:** Fill in Context, Decision, Consequences, and (optionally) Alternatives Considered.
4. **Review:** Open a PR. The ADR is reviewed for clarity, completeness, and consistency.
5. **Accept:** On approval, the ADR's status changes from `proposed` to `accepted` and is merged.

### Three sources of status must agree

An ADR states its lifecycle state in three places, and they were allowed to drift:

1. the `status:` frontmatter key (the machine-readable one, validated against the enum),
2. the `## Status` heading in the body (the human-readable one),
3. any banner — blockquote or HTML comment — near the top of the body.

**Rule: all three carry the same value.** When one changes, all three change in the
same commit. A banner that says `proposed` under a `status: accepted` frontmatter is
a bug, not a nuance. When an ADR is superseded, add `superseded_by:` to it and
`supersedes:` to the replacement, and name the replacement in the body `## Status`.

This rule exists because the drift was found, not predicted: at the 2026-09-29 pass
13 of 54 ADRs had a frontmatter value outside the schema enum, and several had a
banner contradicting both.

### Verifying an ADR before writing it

Claims in an ADR are historical records and are **not** rewritten when a later decision
reverses them — a reversed decision is marked `superseded`, not edited. But a claim
that is simply *wrong about the code* is corrected in place with a dated verification
note naming the `file:line` that settles it. The 2026-09-29 pass corrected
`ADR-0005`'s `ef_construction` this way after finding the shipped default in
`src/index/graph/types.rs:139`.

An ADR that was written as `proposed` and has since shipped in full is promoted to
`accepted`, with the code evidence recorded — not left `proposed` forever.

### Technical Debt / Costs

- **Process overhead.** Writing an ADR takes 15–60 minutes per decision. This is a deliberate investment: the cost of not documenting is paid repeatedly in re-debate and misalignment.
- **Maintenance burden.** ADRs must be reviewed periodically. Freshness is derived from git rather than from a self-reported `last_reviewed` date (see the `$comment` in `docs/_schema/frontmatter.schema.json`); a hand-maintained date is an opinion, not a measurement. Stale ADRs should be revisited or explicitly deprecated.
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
