## Summary

Describe what changed and why.

## Scope

- [ ] Product code
- [ ] Tests
- [ ] Documentation
- [ ] CI / release tooling
- [ ] Packaging

## Validation

List the commands or workflows used to validate this change.

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --profile audit --workspace
```

## Risk

Describe compatibility, data, performance, or release risks. If none are expected, say why.

## Notes

Mention any follow-up work, deferred checks, or heavy certification runs.
