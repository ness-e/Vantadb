# unified-review

> Universal review, audit, and certification skill for OpenCode. Replaces
> `vantadb-full-review`, `vantadb-certify`, and `vantadb-audit` with a
> single skill that works on **any** software project (Rust, Python, TS,
> Go, mixed) and ships with a VantaDB-specific profile.

## What's in this folder

```
unified-review/
├── SKILL.md                      ← the skill (install this in OpenCode)
├── ARCHITECTURE.md               ← design doc (sub-agent flow, data contracts, failure handling)
├── README.md                     ← this file
├── profiles/
│   ├── default.yml               ← generic profile (any project)
│   └── vantadb.yml               ← VantaDB profile (inherits default, adds VantaDB specifics)
└── templates/
    └── pre-push.ps1.tmpl         ← PowerShell pre-push hook (referenced by vantadb.yml)
```

## Installation (OpenCode)

1. **Copy the folder** to your OpenCode skills directory:

   ```
   .opencode/skills/unified-review/
   ```

   (Either project-local `.opencode/skills/` or global `~/.config/opencode/skills/`.)

2. **Verify the skill loads**: start OpenCode and check that
   `unified-review` appears in the `skill` tool's `<available_skills>` list.
   If it doesn't, check:
   - `SKILL.md` is in all caps.
   - Frontmatter has `name` and `description`.
   - `name` (`unified-review`) matches the directory name.

3. **(Optional) Define custom subagent types**. The VantaDB profile
   references `vanta-worker`, `vanta-audit`, `vanta-tuner`, `vanta-docs`,
   `vanta-arch`, `vanta-lead`. If you don't define them, the orchestrator
   falls back to the built-in `general` subagent. To get the full VantaDB
   experience, create markdown files in `.opencode/agents/` — see
   `ARCHITECTURE.md → Section 7` for ready-to-use definitions.

4. **(Optional) Configure permissions**. Add this to `opencode.json`:

   ```json
   {
     "permission": {
       "bash": "allow",
       "read": "allow",
       "edit": "ask",
       "write": "allow",
       "task": "allow",
       "skill": "allow"
     },
     "agent": {
       "build": {
         "permission": {
           "task": {
             "*": "deny",
             "general": "allow",
             "explore": "allow",
             "scout": "allow",
             "vanta-*": "allow"
           }
         }
       }
     }
   }
   ```

## Quick start

### Generic project (any language)

```
# Quick: did I break anything?
/review quick

# Pre-push gate
/review certify

# PR review
/review review

# Quarterly deep dive
/review full
```

### VantaDB

```
# Pre-push certify (matches legacy vantadb-certify)
/review certify --profile vantadb

# Full quarterly review (matches legacy vantadb-full-review)
/review full --profile vantadb

# Legacy alias (matches legacy vantadb-audit)
/audit              # → /review full
/audit quick        # → /review quick
/audit certify      # → /review certify
/audit review       # → /review review
/audit full         # → /review full
```

## Migration from legacy skills

| Old | New |
|-----|-----|
| `vantadb-full-review` skill | `/review full --profile vantadb` |
| `vantadb-certify` skill | `/review certify --profile vantadb` |
| `vantadb-audit` skill | this skill (`/audit` is now an alias) |
| `/audit` | `/review full` (alias kept) |
| `/audit quick` | `/review quick` |
| `/audit certify` | `/review certify` |
| `/audit review` | `/review review` |
| `/audit full` | `/review full` |

The three legacy skills have been **removed from git tracking** (`git rm --cached`)
but their files remain on disk in git history. All active references across
`.opencode/` and `.antigravity/` have been migrated to use `unified-review`.
The legacy files are kept only for history — do not load or invoke them.

## What you get vs the legacy skills

| Capability | Legacy | Unified |
|-----------|--------|---------|
| Compile + lint + tests | ✅ | ✅ (in L1 sub-agent) |
| Security audit | ✅ | ✅ (L7, vanta-audit sub-agent) |
| Performance audit | ✅ | ✅ (L8, vanta-tuner sub-agent) |
| Code review with veto | ✅ | ✅ (L9, vanta-audit sub-agent) |
| CI/CD parity check | ✅ (L7a in certify) | ✅ (L4, vanta-lead sub-agent) |
| Docs coverage | ✅ | ✅ (L5, vanta-docs sub-agent) |
| Architecture review | ✅ | ✅ (L6, vanta-arch sub-agent) |
| ISO 25010 + SonarQube + CII + OWASP + CodeClimate scoring | ✅ (full-review only) | ✅ (when scoring.enabled in profile) |
| 12-category findings taxonomy | ✅ (full-review F9) | ✅ (in every profile) |
| PowerShell pre-push hook | ✅ (certify) | ✅ (template + generator) |
| Codegraph impact analysis | ✅ (certify L0) | ✅ (L0 in default + VantaDB) |
| Ponytail integration | partial | ✅ (respects off/lite/full/ultra modes) |
| Campaign task system integration | ✅ (audit only) | ✅ (in every profile, auto-detected) |
| Parallel execution | ❌ (sequential monolithic) | ✅ (fan-out with task tool, max 4 parallel) |
| Context budget | ❌ (often truncates on VantaDB) | ✅ (< 10% orchestrator, 15-20% per sub-agent) |
| Works on non-VantaDB projects | ❌ (hardcoded paths) | ✅ (default profile auto-detects) |

## Customizing for your project

1. Create `profiles/<myproject>.yml` next to `default.yml`.
2. Set `profile.inherits: default` (or `vantadb` if you want a VantaDB-like base).
3. Override only what differs (commands, paths, thresholds, subagent types).
4. Run `/review <mode> --profile myproject`.

See `default.yml` and `vantadb.yml` for the full schema with examples.

## Documentation

- `SKILL.md` — full skill reference (entry points, detection engine, phase
  catalog, fan-out pattern, findings taxonomy, scoring, report format,
  integrations, usage examples).
- `ARCHITECTURE.md` — design doc (sub-agent flow diagrams, data contracts,
  failure handling, context budget analysis, subagent type definitions,
  extension points, testing guide).
- `profiles/default.yml` — generic profile with extensive inline comments.
- `profiles/vantadb.yml` — VantaDB profile with all VantaDB-specific
  commands, paths, scoring thresholds, and subagent type mappings.

## License

MIT. See `SKILL.md` frontmatter.

## Version

1.0.0 — 2026-07-26
