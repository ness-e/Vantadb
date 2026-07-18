---
description: Start spec-driven development — write a structured specification before writing code
---

Invoke the agent-skills:spec-driven-development skill.
If requirements are ambiguous, also invoke the agent-skills:interview-me skill to extract what the user actually needs.

Begin by understanding what the user wants to build. Ask clarifying questions about:
1. The objective and target users
2. Core features and acceptance criteria
3. Tech stack preferences and constraints
4. Known boundaries (what to always do, ask first about, and never do)

Then generate a structured spec covering all six core areas: objective, commands, project structure, code style, testing strategy, and boundaries.

Save the spec as `SPEC.md` (repo root) or `docs/SPEC.md`. Also consider creating `docs/architecture/adr/` for architectural decisions.
After writing the spec, registrá la decisión en la memoria del campaign-executor:
- `campaign_memory_write(file="decisions", entry="Spec: {nombre} — {decisión clave}")` (MCP)
- También podés consultar `campaign_memory_read(file="decisions")` para ver decisiones anteriores.

**Decision gates before writing:**
- Is the objective clear enough to write testable acceptance criteria? If not, interview the user.
- Are there existing specs, ADRs, or design docs to build on? Read them first.
- Does the tech stack need validation? Run `/audit quick` after spec is written.

## Output format

```markdown
# Spec: [Project/Feature Name]

## Objective
[1-2 lines]

## Target Users
[who, what problem]

## Core Features
- [ ] Feature 1: [AC: what "done" looks like]

## Tech Stack
[language, framework, deps, constraints]

## Boundaries
- Always: [patterns to follow]
- Ask first: [decisions requiring approval]
- Never: [prohibited patterns]

## Testing Strategy
[how to verify each feature]

## Project Structure
[file tree or module layout if known]
```
