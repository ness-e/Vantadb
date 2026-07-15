---
id: "{{ID}}"
name: "{{NAME}}"
created: "{{DATE}}"
module: "{{MODULE}}"
status: "definition"
estimate: "{{ESTIMATE}} turns"
---

# {{ID}}: {{NAME}}

## Contract
```
Verifiable condition when this task is complete.
```

## Atomic Steps
1. **Step 1** — description
2. **Step 2** — description
3. **Step 3** — description

## Skills
- `source-driven-development`
- `ponytail full`

## Checks
- `cargo build`
- `cargo nextest run --profile audit --workspace --build-jobs 2`

## Blast Radius
- **Files:** `path/to/file.rs:1-100`
- **Modules upstream:** (dependen de esto)
- **Modules downstream:** (de lo que esto depende)
- **API changes:** none/internal/public

## Investigation Notes
- CodeGraph findings
- Web research (if needed)
- Design decisions

## AGENTS.md Updates
- Learning to persist after completion
