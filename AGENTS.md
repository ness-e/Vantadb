# VantaDB

Project-level instructions are in `.opencode/AGENTS.md` — this file exists for compatibility with agents (e.g. Claude Code) that look for `AGENTS.md` at the repo root.

For OpenCode, `.opencode/AGENTS.md` is the single source of truth for agent instructions, including CodeGraph usage.

## Release Workflow (Regla 7)

**main → releases, develop → trabajo diario.** Release-plz automatiza versionado y publicación.

### Conventional Commits (obligatorio)
- `feat:` → minor, `fix:` → patch, `docs:|test:|perf:|refactor:` → patch
- `feat!:`, `BREAKING CHANGE:` → major
- `ci:|chore:` → no release
- **NUNCA** tocar versión en Cargo.toml, CHANGELOG, o tags manualmente

### Flujo
```
develop → commit → PR → merge a main → release-plz → Release PR → merge → publish
```
Ver `.opencode/AGENTS.md` → Regla 7 para detalles completos.
