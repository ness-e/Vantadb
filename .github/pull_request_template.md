## Base: `develop` (never `main` directly)

## Message (Conventional Commits — drives releases via release-plz)
- `feat:` → minor release · `fix:` → patch · `!` → major
- `docs:/test:/perf:/refactor:/chore:` → no release

## Gate (must be green)
- [ ] `just verify` (fmt + clippy + nextest + deny) or the same CI checks

## Rules
- [ ] No version bumps, no `CHANGELOG.md` edits, no manual tags (release-plz owns them)
- [ ] No new warnings; focused on a single concern
