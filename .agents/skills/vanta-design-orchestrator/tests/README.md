# Vanta Design Orchestrator — Tests

## Test suite

```bash
node tests/validate-routes.test.mjs
```

### What is tested

1. All 14 layer files exist in `layers/`
2. `routes.json` is valid JSON with correct structure
3. All routes have valid task, pipeline, and priority
4. No duplicate task names across all categories
5. Route count matches the declared `total_routes`
6. All 16 strategy doc files exist in `strategy/`
7. All 3 workflow files are valid JSON with phases
8. Conflict resolutions reference valid skills
9. SKILL.md has valid YAML frontmatter
10. `project-presets.json` is valid
11. `routes-schema.json` is valid JSON Schema
12. SKILL.md references layer files and new CLI scripts
13. Core scripts exist and are parseable JavaScript
14. Multi-platform scripts exist (.mjs, not just .ps1)
15. Layer files are non-empty

### CI integration

Add to your CI pipeline:

```bash
node .agents/skills/vanta-design-orchestrator/tests/validate-routes.test.mjs
node .agents/skills/vanta-design-orchestrator/scripts/validate-routes.mjs
```
