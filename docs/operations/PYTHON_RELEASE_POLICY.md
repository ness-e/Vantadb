---
title: "Python Release Policy & Wheel Engineering"
type: operations
status: active
tags: [vantadb, operations]
last_reviewed: 2026-07-01
---

# Python Release Policy & Wheel Engineering

This document outlines the standard release policy and engineering practices for the VantaDB Python SDK (`vantadb-python`).

## Release Scope

The repository is prepared for Python wheel publishing, but this task does not
publish to PyPI. Production publication requires a signed `vMAJOR.MINOR.PATCH`
tag and a configured PyPI Trusted Publisher or equivalent repository token.

## PEP-440 Version Compliance

All releases of `vantadb_py` must strictly adhere to the [PEP-440 specification](https://peps.python.org/pep-0440/).

### Versioning Format

VantaDB uses the standard semantic versioning format for production releases:
`MAJOR.MINOR.PATCH` (e.g., `0.1.1`)

For pre-releases, the following formats are permitted:

- **Alpha:** `MAJOR.MINOR.PATCHaN` (e.g., `0.1.2a1`)
- **Beta:** `MAJOR.MINOR.PATCHbN` (e.g., `0.1.2b1`)
- **Release Candidate:** `MAJOR.MINOR.PATCHrcN` (e.g., `0.1.2rc1`)

### Tagging & Release Sync

- Every Python SDK release must be associated with a Git tag matching `vMAJOR.MINOR.PATCH`.
- `Cargo.toml`, `vantadb-python/Cargo.toml`, and
  `vantadb-python/pyproject.toml` must all match the release tag exactly.
- `cargo test --test version_coherence --all-features` is the local guardrail
  for version drift before building wheels.

---

## Supply-Chain Integrity via GitHub Attestations (SLSA Level 2)

To ensure supply-chain integrity, all built Python wheels (`.whl`) are signed using
[GitHub Attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations)
(`actions/attest-build-provenance@v4`), which generates cryptographically signed
SLSA Level 2 provenance records bound to the GitHub Actions OIDC identity.

### Why GitHub Attestations over Sigstore Standalone

| Mechanism | Status |
|---|---|
| `sigstore/gh-action-sigstore-python` | Superseded — GitHub Attestations provides equivalent guarantees natively |
| `actions/attest-build-provenance@v4` | **Active** — SLSA Level 2, integrated with GitHub OIDC, no Fulcio/Rekor self-management |

### Attestation Flow

1. The release workflow uses GitHub Actions OIDC (`id-token: write`) to establish a verified identity.
2. `actions/attest-build-provenance@v4` generates a signed SLSA provenance statement for each wheel.
3. The attestation is stored in GitHub's immutable attestation store, linked to the repository.
4. Wheels and their signed GitHub Release assets are published alongside each release.

### Verification Command

Downstream users can verify the integrity of any VantaDB wheel using the `gh` CLI:

```bash
# Download the wheel you want to verify
pip download --no-deps "vantadb-py==0.1.4"

# Verify provenance against the official repository
gh attestation verify vantadb_py-0.1.4-*.whl \
  --repo ness-e/Vantadb
```

A successful verification output confirms:

```
Loaded digest sha256:<digest> for file://vantadb_py-0.1.4-*.whl
✓ Attestation verified — build provenance linked to ness-e/Vantadb
```

> **Note:** The `gh` CLI must be authenticated (`gh auth login`) and have access to the public repository.

---

## Standard Release Flow

### 1. Local preflight

```bash
cargo test --test version_coherence --all-features
python -m pytest vantadb-python/tests/test_sdk.py -v
python -m maturin build --manifest-path ./vantadb-python/Cargo.toml --out ./target/wheels --release
```

Install the newest local wheel in a clean virtual environment and rerun the
Python SDK smoke suite before staging.

### 2. TestPyPI staging

Run the `Python Wheels` workflow manually with `publish_testpypi=true`.

The workflow:
1. Builds wheels on Linux, macOS, and Windows
2. Runs wheel smoke tests on all three platforms
3. Uploads the merged wheel set to TestPyPI
4. **[Automated]** The `verify-testpypi-install` job downloads the wheel from TestPyPI
   and validates import + version match — this is the gate before production.

Validate manually in a clean environment:

```bash
python -m pip install \
  --index-url https://test.pypi.org/simple/ \
  --extra-index-url https://pypi.org/simple \
  vantadb-py
python -m pytest vantadb-python/tests/test_sdk.py -v
```

### 3. Production PyPI

Create and push a release tag only after TestPyPI validation:

```bash
git tag -s v0.1.4 -m "VantaDB Python 0.1.4"
git push origin v0.1.4
```

Tag pushes trigger the production pipeline which:
1. Builds and attests wheels via `actions/attest-build-provenance@v4`
2. Attaches `.whl` assets to the GitHub Release
3. Publishes to PyPI via Trusted Publishing (OIDC — no tokens needed)
4. **[Automated]** The `verify-pypi-install` job waits 90s for CDN propagation,
   installs from PyPI production, runs a smoke test, and verifies the attestation
   via `gh attestation verify`.

### 4. Rollback

PyPI files are immutable. If a release is bad:

- Yank the affected release on PyPI instead of deleting files.
- Publish a patched version with a higher patch or post-release version.
- Leave GitHub Release assets intact and add a release note with the yanked
  reason and replacement version.

---

## PyPI Fallback Rules

In the event of network disruption, token expiration, or service outages on primary package registries, the following fallback guidelines apply:

### 1. TestPyPI Staging Validation

- Before releasing to production PyPI, wheels must be built and validated against **TestPyPI**.
- Manual triggering via `workflow_dispatch` is available to test builds on TestPyPI with:

  ```bash
  publish_testpypi: true
  ```

### 2. Manual Fallback Releases

If the automated OIDC publishing fails due to registry maintenance:

- Release engineers may build wheels locally inside isolated Docker containers to guarantee reproducible builds across OS targets.
- Publishing must be done using `twine`:

  ```bash
  python -m twine upload --repository pypi target/wheels/*
  ```

- Manually generated wheels will not have automated attestations. Add a release note
  clarifying the manual build and include the SHA256 checksums computed locally.

### 3. Source Fallback (Direct path / git URL)

In critical rescue scenarios where PyPI is offline, the SDK can be installed directly from the GitHub repository source using Maturin's in-place compilation fallback:

```bash
pip install git+https://github.com/ness-e/Vantadb.git#subdirectory=vantadb-python
```

This requires local installation of the Rust toolchain on the target system.

---

## CI Pipeline Summary

```
push v*.*.* tag
       │
       ▼
  [gate] version_coherence
       │
       ▼
  [build-wheels] Linux / macOS / Windows
  (smoke test per platform)
       │
       ├──────────────────────────┐
       ▼                          ▼
  [publish-pypi]            [publish-testpypi]
  + attest-build-provenance  (on main / dispatch)
  + attach to GitHub Release       │
       │                           ▼
       ▼                  [verify-testpypi-install]
  [verify-pypi-install]   import smoke + version check
  90s CDN wait
  import smoke + version check
  gh attestation verify ← SLSA Level 2 gate
```
