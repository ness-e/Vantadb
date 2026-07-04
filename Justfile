# VantaDB — Justfile (cross-platform task runner)
# Install: `cargo install just`
# Usage:   `just check` `just test` `just watch` `just verify`

cargo := "cargo"
nextest_args := "--workspace"

# List available commands
default:
    @just --list

# Check code compiles (fast)
check:
    {{cargo}} check --workspace

# Run clippy lints (deny warnings)
clippy:
    {{cargo}} clippy --workspace --all-targets --all-features -- -D warnings

# Format code (check only)
fmt:
    {{cargo}} fmt --check

# Format code (apply fixes)
fmt-fix:
    {{cargo}} fmt

# Run nextest audit profile
test:
    {{cargo}} nextest run --profile audit {{nextest_args}} --build-jobs 2

# Run a specific test
test-one test_name:
    {{cargo}} nextest run --profile audit {{nextest_args}} --test {{test_name}} --build-jobs 2

# Run all tests (slow - includes heavy)
test-all:
    {{cargo}} nextest run --workspace --build-jobs 2

# Run experimental tests
test-experimental:
    {{cargo}} nextest run --profile experimental {{nextest_args}}

# Full pre-flight verification
verify: fmt clippy test deny

# Quick verify (CodeGraph-optimized, ~30s)
verify-quick:
    pwsh -NoProfile -File dev-tools/verify_changed.ps1

# Security audit
deny:
    {{cargo}} deny check

# Audit advisories only
audit:
    {{cargo}} audit

# Watch for changes (check + test)
watch:
    {{cargo}} watch -x "check --workspace" -x "nextest run --profile audit {{nextest_args}} --build-jobs 2"

# Watch: check only (fastest)
watch-check:
    {{cargo}} watch -x "check --workspace"

# Unused dependency check
machete:
    {{cargo}} machete

# Check outdated dependencies
outdated:
    {{cargo}} outdated --exit-code 1

# Binary size analysis
size:
    {{cargo}} bloat --crates

# Clean build artifacts
clean:
    {{cargo}} clean

# Build release
release:
    {{cargo}} build --release

# Run vanta-cli
run-cli:
    {{cargo}} run --features cli

# Run vanta server
run-server:
    {{cargo}} run --features server --bin vantadb-server

# Run MCP server
run-mcp:
    {{cargo}} run -p vantadb-mcp

# Setup Python venv + build SDK
python-setup:
    pwsh -NoProfile -File dev-tools/setup_venv.ps1

# Run Python SDK tests
python-test:
    pwsh -NoProfile -File dev-tools/scripts/validate_python_sdk.ps1

# Generate changelog
changelog:
    git-cliff -o docs/CHANGELOG.md

# Full CI suite (what CI runs)
ci: fmt clippy test deny audit

# Heavy certification suite (runs locally)
certify:
    pwsh -NoProfile -File dev-tools/nocturnal_suite.ps1

# Documentation coverage check
docs:
    pwsh -NoProfile -File scripts/validate-docs-coverage.ps1

# Collect code snapshot for AI context
code-snapshot:
    pwsh -NoProfile -File dev-tools/scripts/collect_code.ps1
