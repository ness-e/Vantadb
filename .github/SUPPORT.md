# Support

Use the channel that matches the kind of help you need.

## Bugs

Open a bug report with a minimal reproduction, version or commit SHA, operating system, and relevant
logs. Remove private data and local database contents before posting.

## Feature Requests

Open a feature request when the change fits VantaDB's current local-first embedded memory and
retrieval boundary.

## Documentation

Open a documentation issue for stale instructions, broken links, unclear examples, or mismatched
product claims.

## Security

Do not report sensitive vulnerabilities in a public issue. Follow [SECURITY.md](SECURITY.md) and use
GitHub Security Advisories if private reporting is enabled.

## Local Validation

Before asking for debugging help, include the result of the smallest relevant command:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --profile audit --workspace
python -m pytest vantadb-python/tests/test_sdk.py -q
```
